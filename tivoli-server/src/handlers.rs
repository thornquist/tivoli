use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;

use crate::db::InMemoryDb;
use crate::errors::AppError;
use crate::models::*;
use crate::queries;

pub struct AppState {
    pub db: InMemoryDb,
    pub galleries_path: std::path::PathBuf,
    pub thumbnail_cache_dir: std::path::PathBuf,
}

pub async fn search_images(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<Vec<ImageRow>>, AppError> {
    let (sql, params) = queries::build_image_query(&request.filters)?;
    let conn = state.db.conn()?;
    let images = queries::query_images(&conn, &sql, &params)?;
    Ok(Json(images))
}

pub async fn search_filter_options(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<FilterOptions>, AppError> {
    let conn = state.db.conn()?;
    let options = queries::query_filter_options(&conn, &request.filters)?;
    Ok(Json(options))
}

pub async fn get_image_detail(
    State(state): State<Arc<AppState>>,
    Path(uuid): Path<String>,
) -> Result<Json<ImageDetail>, AppError> {
    let conn = state.db.conn()?;
    let detail = queries::query_image_detail(&conn, &uuid)?;
    Ok(Json(detail))
}

pub async fn get_image_file(
    State(state): State<Arc<AppState>>,
    Path(uuid): Path<String>,
    Query(params): Query<ImageFileParams>,
) -> Result<impl IntoResponse, AppError> {
    let full_path = {
        let conn = state.db.conn()?;
        let path: String = conn
            .query_row("SELECT path FROM images WHERE uuid = ?", [&uuid], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AppError::NotFound("Image not found".into())
                }
                other => AppError::from(other),
            })?;
        state.galleries_path.join(&path)
    };
    let canonical = full_path
        .canonicalize()
        .map_err(|_| AppError::NotFound("File not found".into()))?;

    if !canonical.starts_with(&state.galleries_path) {
        return Err(AppError::BadRequest("Invalid path".into()));
    }

    // No width requested — serve full-resolution file
    let Some(target_width) = params.w else {
        let body = tokio::fs::read(&canonical)
            .await
            .map_err(|_| AppError::NotFound("File not found on disk".into()))?;
        return Ok((
            [(axum::http::header::CONTENT_TYPE, "image/jpeg")],
            body,
        ));
    };

    let target_width = target_width.clamp(50, 1920);

    // Check disk cache
    let cache_path = state
        .thumbnail_cache_dir
        .join(format!("{uuid}_{target_width}.jpg"));

    if let Ok(cached) = tokio::fs::read(&cache_path).await {
        return Ok((
            [(axum::http::header::CONTENT_TYPE, "image/jpeg")],
            cached,
        ));
    }

    // Generate thumbnail on blocking thread
    let source_path = canonical.clone();
    let out_path = cache_path.clone();
    let body = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, AppError> {
        let img = image::open(&source_path)
            .map_err(|e| AppError::BadRequest(format!("Failed to decode image: {e}")))?;

        let thumb = if img.width() > target_width {
            img.thumbnail(target_width, u32::MAX)
        } else {
            img
        };

        let mut buf = Vec::new();
        thumb
            .write_to(
                &mut std::io::Cursor::new(&mut buf),
                image::ImageFormat::Jpeg,
            )
            .map_err(|e| AppError::BadRequest(format!("Failed to encode thumbnail: {e}")))?;

        if let Err(e) = std::fs::write(&out_path, &buf) {
            tracing::warn!("Failed to cache thumbnail: {e}");
        }

        Ok(buf)
    })
    .await
    .map_err(|e| AppError::DbError(format!("Thumbnail task failed: {e}")))??;

    Ok((
        [(axum::http::header::CONTENT_TYPE, "image/jpeg")],
        body,
    ))
}

pub async fn update_image_tags(
    State(state): State<Arc<AppState>>,
    Path(uuid): Path<String>,
    Json(request): Json<UpdateTagsRequest>,
) -> Result<impl IntoResponse, AppError> {
    {
        let conn = state.db.conn()?;
        queries::replace_image_tags(&conn, &uuid, &request.tag_uuids)?;
    }
    // Flush to disk in background
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        if let Err(e) = db.flush_to_disk() {
            tracing::error!("Failed to flush DB to disk: {e}");
        }
    });
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn list_collections(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CollectionSummary>>, AppError> {
    let conn = state.db.conn()?;
    let collections = queries::query_collections(&conn)?;
    Ok(Json(collections))
}

pub async fn list_galleries(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<CollectionFilter>,
) -> Result<Json<Vec<GallerySummary>>, AppError> {
    let conn = state.db.conn()?;
    let galleries = queries::query_galleries(&conn, filter.collection.as_deref())?;
    Ok(Json(galleries))
}

pub async fn list_models(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<CollectionFilter>,
) -> Result<Json<Vec<Model>>, AppError> {
    let conn = state.db.conn()?;
    let models = queries::query_models(&conn, filter.collection.as_deref())?;
    Ok(Json(models))
}

pub async fn list_tags(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TagGroup>>, AppError> {
    let conn = state.db.conn()?;
    let groups = queries::query_tag_groups(&conn)?;
    Ok(Json(groups))
}
