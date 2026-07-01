use std::{collections::HashMap, env};

use axum::{
    Json, Router,
    routing::{get, post},
};
use backon::{ExponentialBuilder, Retryable};
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

#[derive(Serialize, Deserialize)]
struct PickSupervisorData {
    pub name: String,
    pub slug: String,
}

#[derive(Serialize, Deserialize)]
struct PickSupervisorResponse {
    pub potential_supervisors: Vec<PickSupervisorData>,
}

static COLLECTION_NAME: &str = "lecturers";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let app = Router::new()
        .route("/healthcheck", get(health_check))
        .route("/supervisors/pick", post(pick_supervisor))
        .layer(CorsLayer::permissive()); // Allow all origins for dev

    let port = env::var("PORT").unwrap_or_else(|_| return "3000".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| return "127.0.0.1".to_string());

    let address = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    println!("Listening on http://{address}");
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<Value> {
    return Json(json!({"status": "ok", "message": "Axum backend is running!"}));
}

static NEAREST_VECTOR_QUERY_TEMPLATE: &str = r"
# STUDENT

## Biography:
{additional_text}

## Area of Expertise (seeking):
{interesting_topics}
";

static QUERY_PROMPT_TEMPLATE: &str = r#"
Context: {context}

Query: Which lecturer, based on the context alone, should be my research supervsior? STRICTLY return TWO of the most promising results following the format from this JSON schema: {response_schema}. The length of `potential_supervisors` should be 2, meaning return 2 of of most promising results.

Do not hallucinate, and do not make any mistake. I believe in you.
"#;

/// Queries the Qdrant database to find the nearest points matching the given
/// text, with exponential backoff retry on failure.
///
/// # Why retry?
///
/// The free Qdrant plan may intermittently timeout queries. Exponential backoff
/// provides resilience by retrying with increasing delays between attempts,
/// giving the server time to recover from transient load spikes.
///
/// # Flow
///
/// 1. Clones the query text so each retry attempt owns its copy.
/// 2. Wraps the Qdrant `.query()` call in a closure.
/// 3. Applies `backon::Retryable` with `ExponentialBuilder::default()`,
///    which starts at a 1-second delay and doubles on each retry.
/// 4. Awaits the retry future; on success, returns the response.
///
/// # Arguments
///
/// * `qdrant_client` - Reference to the already-built Qdrant client.
/// * `openrouter_api_key` - `OpenRouter` API key used for the embedding model.
/// * `text` - The query text for nearest-neighbor search.
///
/// # Returns
///
/// The raw `QueryResponse` from Qdrant.
///
/// # Panics
///
/// Panics if all retry attempts are exhausted and the query still fails.
async fn query_qdrant_with_retry(
    qdrant_client: &Qdrant,
    openrouter_api_key: &str,
    text: &str,
) -> qdrant_client::qdrant::QueryResponse {
    // Owned copy placed outside the closure so the closure captures a
    // reference — allowing it to be FnMut (called multiple times).
    let query_text = text.to_owned();
    // The closure captures &Qdrant and &str references, which are Copy.
    // Each invocation creates a new async block that borrows the client and
    // clones the query text for its own attempt.
    return (|| {
        return async {
            let text = query_text.clone();
            return qdrant_client
                .with_header("openrouter-api-key", openrouter_api_key)
                .query(
                    QueryPointsBuilder::new(COLLECTION_NAME)
                        .query(Query::new_nearest(Document {
                            text,
                            model: "openrouter/qwen/qwen3-embedding-8b".into(),
                            options: HashMap::new(),
                        }))
                        .limit(5)
                        .with_payload(true)
                        .build(),
                )
                .await;
        };
    })
    .retry(ExponentialBuilder::default())
    .when(|e| {
        // Retry on every error — timeouts are the primary concern on the free
        // plan, but any transient failure should be retried.
        eprintln!("Qdrant query failed, retrying: {e}");
        return true;
    })
    .await
    .unwrap(); // All retries exhausted — let the panic propagate.
}

async fn pick_supervisor(Json(payload): Json<PickSupervisorRequest>) -> Json<PickSupervisorResponse> {
    let interesting_topics_str: Vec<String> = payload
        .interesting_topics
        .iter()
        .map(|item| format!("- {item}"))
        .collect();
    let interesting_topics_str = interesting_topics_str.join("\n");
    let interesting_topics_str = interesting_topics_str.as_str();
    let text = NEAREST_VECTOR_QUERY_TEMPLATE
        .replacen("{interesting_topics}", interesting_topics_str, 1)
        .replacen("{additional_text}", payload.additional_text.as_str(), 1);

    let cluster_endpoint = env::var("QDRANT_CLUSTER_ENDPOINT")
        .unwrap_or_else(|_| return "http://127.0.0.1:6334".to_string());
    let qdrant_api_key =
        env::var("QDRANT_API_KEY").unwrap_or_else(|_| return "not_needed".to_string());
    let openrouter_base_url =
        env::var("OPENROUTER_BASE_URL").unwrap_or_else(|_| return "not_needed".to_string());
    let openrouter_api_key =
        env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| return "not_needed".to_string());
    // Build the Qdrant client once — this should never fail transiently.
    let qdrant_client = Qdrant::from_url(cluster_endpoint.as_str())
        .api_key(qdrant_api_key)
        .build()
        .unwrap();
    // The actual query is wrapped with exponential backoff via `backon`
    // so intermittent timeouts on the free plan are retried automatically.
    let query_response = query_qdrant_with_retry(&qdrant_client, &openrouter_api_key, &text).await;

    let context: Vec<_> = query_response
        .result
        .iter()
        .map(|item| {
            return item.payload.get("text").unwrap().to_string();
        })
        .collect();
    let context = context.join("\n\n");

    let response_schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "potential_supervisors": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string"
                        },
                        "slug": {
                            "type": "string"
                        }
                    },
                    "required": [
                        "name",
                        "slug"
                    ]
                }
            }
        }
    });
    let query_prompt = QUERY_PROMPT_TEMPLATE
        .replacen("{context}", context.as_str(), 1)
        .replacen(
            "{response_schema}",
            serde_json::to_string(&response_schema).unwrap().as_str(),
            1,
        );
    let request_body = json!({
        "model": "deepseek/deepseek-v4-flash",
        "messages": [
            {
                "role": "system",
                "content": "Answer based only on given context. Do not search the internet or make any tool calls."
            },
            {
                "role": "user",
                "content": query_prompt
            }
        ],
        "response_format": response_schema
    });
    let reqwest_client = reqwest::Client::builder().build().unwrap();
    let request = reqwest_client
        .request(
            reqwest::Method::POST,
            format!("{openrouter_base_url}/chat/completions"),
        )
        .header("Authorization", format!("Bearer {openrouter_api_key}"))
        .json(&request_body);
    let response = request.send().await.unwrap();
    let response_body: PickSupervisorResponse = response.json().await.unwrap();
    return Json(response_body);
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    /// Verifies that the retry mechanism calls the function multiple times when
    /// earlier attempts fail, and eventually returns the successful result.
    #[tokio::test]
    async fn retry_on_failure_succeeds_eventually() {
        let call_count = AtomicUsize::new(0);

        let result: Result<&str, &str> = (|| async {
            let count = call_count.fetch_add(1, Ordering::SeqCst);
            // Fail on the first two attempts, succeed on the third.
            if count < 2 {
                Err("timeout")
            } else {
                Ok("success")
            }
        })
        .retry(ExponentialBuilder::default())
        .await;

        assert_eq!(result.unwrap(), "success");
        // Should have been called 3 times: 2 failures + 1 success.
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    /// Verifies that no retries are performed when the function succeeds on the
    /// first attempt.
    #[tokio::test]
    async fn retry_stops_on_first_success() {
        let call_count = AtomicUsize::new(0);

        let result: Result<&str, &str> = (|| async {
            call_count.fetch_add(1, Ordering::SeqCst);
            Ok("immediate_success")
        })
        .retry(ExponentialBuilder::default())
        .await;

        assert_eq!(result.unwrap(), "immediate_success");
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }
}
