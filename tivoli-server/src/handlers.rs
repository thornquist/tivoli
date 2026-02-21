use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;

use crate::errors::AppError;
use crate::models::*;
use crate::queries;

pub struct AppState {
    pub pool: crate::db::Pool,
    pub galleries_path: std::path::PathBuf,
}

pub async fn search_images(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<Vec<ImageRow>>, AppError> {
    let (sql, params) = queries::build_image_query(&request.filters)?;
    let conn = state.pool.get()?;
    let images = queries::query_images(&conn, &sql, &params)?;
    Ok(Json(images))
}

pub async fn search_filter_options(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<FilterOptions>, AppError> {
    let conn = state.pool.get()?;
    let options = queries::query_filter_options(&conn, &request.filters)?;
    Ok(Json(options))
}

pub async fn get_image_file(
    State(state): State<Arc<AppState>>,
    Path(uuid): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pool.get()?;
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

    let full_path = state.galleries_path.join(&path);
    let canonical = full_path
        .canonicalize()
        .map_err(|_| AppError::NotFound("File not found".into()))?;

    if !canonical.starts_with(&state.galleries_path) {
        return Err(AppError::BadRequest("Invalid path".into()));
    }

    let body = tokio::fs::read(&canonical)
        .await
        .map_err(|_| AppError::NotFound("File not found on disk".into()))?;

    Ok((
        [(axum::http::header::CONTENT_TYPE, "image/jpeg")],
        body,
    ))
}

pub async fn list_collections(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CollectionSummary>>, AppError> {
    let conn = state.pool.get()?;
    let collections = queries::query_collections(&conn)?;
    Ok(Json(collections))
}

pub async fn list_galleries(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<CollectionFilter>,
) -> Result<Json<Vec<GallerySummary>>, AppError> {
    let conn = state.pool.get()?;
    let galleries = queries::query_galleries(&conn, filter.collection.as_deref())?;
    Ok(Json(galleries))
}

pub async fn list_models(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<CollectionFilter>,
) -> Result<Json<Vec<Model>>, AppError> {
    let conn = state.pool.get()?;
    let models = queries::query_models(&conn, filter.collection.as_deref())?;
    Ok(Json(models))
}

pub async fn list_tags(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TagGroup>>, AppError> {
    let conn = state.pool.get()?;
    let groups = queries::query_tag_groups(&conn)?;
    Ok(Json(groups))
}
