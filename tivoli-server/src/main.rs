use tivoli_server::build_app;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("TIVOLI_DB_PATH")
        .unwrap_or_else(|_| "../data/tivoli.db".to_string());
    let galleries_dir = std::env::var("TIVOLI_GALLERIES_PATH")
        .unwrap_or_else(|_| "../galleries".to_string());

    let app = build_app(&db_path, &galleries_dir);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
