use std::env;

use axum::{Router, routing::get, Json};
use tower_http::cors::CorsLayer;
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/health", get(health_check))
        .layer(CorsLayer::permissive()); // Allow all origins for dev

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    // Default to 127.0.0.1 for local safety, swap to 0.0.0.0 via env var in production
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let address = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    println!("Listening on http://{}", address);
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<Value> {
    Json(json!({"status": "ok", "message": "Axum backend is running!"}))
}
