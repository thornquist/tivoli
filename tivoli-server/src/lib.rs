mod db;
mod errors;
mod handlers;
mod models;
mod queries;

use std::sync::Arc;

use axum::routing::{get, post, put};
use axum::Router;
use handlers::AppState;

pub fn build_app(db_path: &str, galleries_dir: &str) -> Router {
    let db = db::InMemoryDb::load_from_disk(db_path);

    let galleries_path =
        std::fs::canonicalize(galleries_dir).expect("galleries directory not found");

    let thumbnail_cache_dir = galleries_path.join(".thumbnails");
    std::fs::create_dir_all(&thumbnail_cache_dir).expect("failed to create thumbnail cache dir");

    let state = Arc::new(AppState {
        db,
        galleries_path,
        thumbnail_cache_dir,
    });

    Router::new()
        .route("/images/search", post(handlers::search_images))
        .route("/images/search/options", post(handlers::search_filter_options))
        .route("/images/{uuid}", get(handlers::get_image_detail))
        .route("/images/{uuid}/file", get(handlers::get_image_file))
        .route("/images/{uuid}/tags", put(handlers::update_image_tags))
        .route("/collections", get(handlers::list_collections))
        .route("/galleries", get(handlers::list_galleries))
        .route("/models", get(handlers::list_models))
        .route("/tags", get(handlers::list_tags))
        .with_state(state)
        .layer(tower_http::compression::CompressionLayer::new())
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(tower_http::trace::TraceLayer::new_for_http())
}
