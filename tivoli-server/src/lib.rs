mod db;
mod errors;
mod handlers;
mod models;
mod queries;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use handlers::AppState;

pub fn build_app(db_path: &str, galleries_dir: &str) -> Router {
    let manager = r2d2_sqlite::SqliteConnectionManager::file(db_path);
    let pool = r2d2::Pool::builder()
        .max_size(4)
        .build(manager)
        .expect("Failed to create DB pool");

    let galleries_path =
        std::fs::canonicalize(galleries_dir).expect("galleries directory not found");

    let state = Arc::new(AppState {
        pool,
        galleries_path,
    });

    Router::new()
        .route("/images/search", post(handlers::search_images))
        .route("/images/search/options", post(handlers::search_filter_options))
        .route("/images/{uuid}/file", get(handlers::get_image_file))
        .route("/collections", get(handlers::list_collections))
        .route("/galleries", get(handlers::list_galleries))
        .route("/models", get(handlers::list_models))
        .route("/tags", get(handlers::list_tags))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(tower_http::trace::TraceLayer::new_for_http())
}
