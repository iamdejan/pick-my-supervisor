use std::{collections::HashMap, env};

use axum::{Json, Router, routing::get};
use qdrant_client::{
    Qdrant,
    qdrant::{Document, Query, QueryPointsBuilder},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;

#[derive(Serialize, Deserialize)]
struct PickSupervisorRequest {
    pub interesting_topics: Vec<String>,
    pub additional_text: String,
}

static COLLECTION_NAME: &'static str = "lecturers";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/supervisors/pick", get(pick_supervisor))
        .layer(CorsLayer::permissive()); // Allow all origins for dev

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let address = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    println!("Listening on http://{}", address);
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<Value> {
    Json(json!({"status": "ok", "message": "Axum backend is running!"}))
}

static TEMPLATE: &'static str = r#"
Area of interest:
{}

Additional text:
{}

Based on the given information, who is the most suitable supervisor?
"#;

async fn pick_supervisor(Json(payload): Json<PickSupervisorRequest>) -> Json<Value> {
    let interesting_topics_str: Vec<String> = payload
        .interesting_topics
        .iter()
        .map(|item| format!("- {}", item))
        .collect();
    let interesting_topics_str = interesting_topics_str.join("\n");
    let interesting_topics_str = interesting_topics_str.as_str();
    let text = TEMPLATE.replacen("{}", interesting_topics_str, 1).replacen(
        "{}",
        payload.additional_text.as_str(),
        1,
    );

    let cluster_endpoint =
        env::var("QDRANT_CLUSTER_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:6334".to_string());
    let qdrant_api_key = env::var("QDRANT_API_KEY").unwrap_or_else(|_| "not_needed".to_string());
    let openrouter_api_key =
        env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| "not_needed".to_string());
    let qdrant_client = Qdrant::from_url(cluster_endpoint.as_str())
        .api_key(qdrant_api_key)
        .build()
        .unwrap();
    let query_response = qdrant_client
        .with_header("openrouter-api-key", openrouter_api_key)
        .query(
            QueryPointsBuilder::new(COLLECTION_NAME)
                .query(Query::new_nearest(Document {
                    text: text,
                    model: "openrouter/qwen/qwen3-embedding-8b".into(),
                    options: HashMap::new(),
                }))
                .build(),
        )
        .await
        .unwrap();
    return Json(json!({"result": format!("{:?}", query_response)}));
}
