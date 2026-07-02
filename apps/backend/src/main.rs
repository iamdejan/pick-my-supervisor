use std::{collections::HashMap, env};

use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
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

#[derive(Serialize, Deserialize)]
struct ChatCompletionMessage {
    pub content: String,
    pub role: String,
}

#[derive(Serialize, Deserialize)]
struct ChatCompletionChoice {
    pub message: ChatCompletionMessage,
}

#[derive(Serialize, Deserialize)]
struct ChatCompletionResponse {
    pub choices: Vec<ChatCompletionChoice>,
}

/// Represents an error payload returned by the `OpenRouter` API when a request
/// fails with a non-2xx status code. The `metadata` field captures additional
/// provider-specific information such as the raw upstream error body.
#[derive(Serialize, Deserialize)]
struct OpenRouterError {
    pub message: String,
    pub code: i32,
    #[serde(default)]
    pub metadata: Option<Value>,
}

/// Top-level error envelope returned by `OpenRouter` on non-2xx responses.
#[derive(Serialize, Deserialize)]
struct OpenRouterErrorResponse {
    pub error: OpenRouterError,
}

/// Aggregates all possible error modes the `pick_supervisor` handler can
/// encounter — from external API failures to internal deserialization bugs.
///
/// Each variant maps to a distinct HTTP status code and a JSON error body so
/// the frontend can programmatically distinguish between transient upstream
/// issues and permanent client/developer errors.
#[derive(Debug)]
enum AppError {
    /// The external `OpenRouter` API rejected the request or encountered an
    /// internal failure. The wrapped status code is the upstream HTTP code.
    OpenRouter {
        status: StatusCode,
        message: String,
        code: i32,
        metadata: Option<Value>,
    },
    /// A well-formed HTTP response was received but the body could not be
    /// deserialized. This typically indicates a schema mismatch between the
    /// server and our `ChatCompletionResponse` / `PickSupervisorResponse`
    /// structs.
    Deserialization { context: String, detail: String },
    /// The LLM returned a message with zero choices, which is a provider edge
    /// case but not a parse error.
    EmptyChoices,
    /// Any other internal failure with no upstream HTTP status to reflect.
    Internal(String),
}

/// Converts every `AppError` variant into an Axum HTTP response with an
/// appropriate status code and a JSON-encoded error payload.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            // Forward the upstream status code along with the structured error so
            // the client can react differently to 400 vs 429 vs 502, etc.
            AppError::OpenRouter {
                status,
                message,
                code,
                metadata,
            } => {
                let mut error_body = json!({
                    "error": {
                        "message": message,
                        "code": code
                    }
                });
                if let Some(meta) = metadata {
                    error_body["error"]["metadata"] = meta;
                }
                (status, error_body)
            }
            // Deserialization failures are *our* bug or a provider API change —
            // the frontend should treat them as an internal server error.
            AppError::Deserialization { context, detail } => {
                let error_body = json!({
                    "error": {
                        "message": format!("failed to deserialize {context}: {detail}")
                    }
                });
                (StatusCode::INTERNAL_SERVER_ERROR, error_body)
            }
            AppError::EmptyChoices => {
                let error_body = json!({
                    "error": {
                        "message": "LLM returned zero choices — upstream may have blocked the response"
                    }
                });
                (StatusCode::INTERNAL_SERVER_ERROR, error_body)
            }
            AppError::Internal(detail) => {
                let error_body = json!({
                    "error": {
                        "message": detail
                    }
                });
                (StatusCode::INTERNAL_SERVER_ERROR, error_body)
            }
        };

        let body_str = serde_json::to_string(&body).unwrap_or_else(|_| {
            return r#"{"error":{"message":"failed to serialize error response"}}"#.to_string();
        });
        return (status, body_str).into_response();
    }
}

static COLLECTION_NAME: &str = "lecturers";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let app = Router::new()
        .route("/healthcheck", get(health_check))
        .route("/supervisors/pick", post(pick_supervisor))
        .layer(CorsLayer::permissive());

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

static QUERY_PROMPT_TEMPLATE: &str = r"
Context:
{context}

Query: Which lecturer, based on the context alone, should be my research supervsior? STRICTLY return TWO of the most promising results following the format from this JSON schema: {response_schema}. The length of `potential_supervisors` should be 2, meaning return 2 of of most promising results.

Do not hallucinate, and do not make any mistake. I believe in you.
";

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

/// Sends a chat-completion request to the `OpenRouter` API, checks the HTTP
/// status code, and parses the response body.
///
/// # Why this is a separate function
///
/// The request body construction, HTTP call, status-code gating, and chat-
/// completion deserialization collectively represent a single responsibility:
/// "call the LLM and get back a structured `ChatCompletionResponse`."
/// Extracting it keeps `pick_supervisor` focused on orchestration.
///
/// # Arguments
///
/// * `openrouter_base_url` - Base URL for the `OpenRouter` API.
/// * `openrouter_api_key` - `OpenRouter` API key.
/// * `query_prompt` - The assembled user prompt (context + JSON schema).
///
/// # Returns
///
/// The `ChatCompletionResponse` on success.
///
/// # Errors
///
/// `AppError::OpenRouter` on non-2xx upstream status. `AppError::Deserialization`
/// when the 2xx body cannot be parsed. `AppError::EmptyChoices` when the LLM
/// returns zero choices. `AppError::Internal` on network/HTTP-client errors.
async fn send_openrouter_chat_completion(
    openrouter_base_url: &str,
    openrouter_api_key: &str,
    query_prompt: &str,
) -> Result<ChatCompletionResponse, AppError> {
    // `response_format` must use the API-supported format. DeepSeek (and
    // OpenAI-compatible providers) expect `{"type": "json_object"}` — sending
    // a raw JSON Schema object with `"type": "object"` at the root is rejected
    // because the provider interprets `response_format.type` and sees `object`,
    // which is not a recognised variant (valid: json_object, json_schema,
    // regex, text).
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
        "response_format": {
            "type": "json_object"
        },
        "provider": {
            "only": ["deepseek"],
            "allow_fallbacks": false
        }
    });
    let reqwest_client = reqwest::Client::builder().build().map_err(|e| {
        return AppError::Internal(format!("failed to build HTTP client: {e}"));
    })?;
    let request = reqwest_client
        .request(
            reqwest::Method::POST,
            format!("{openrouter_base_url}/chat/completions"),
        )
        .header("Authorization", format!("Bearer {openrouter_api_key}"))
        .json(&request_body);
    let response = request.send().await.map_err(|e| {
        return AppError::Internal(format!("HTTP request to OpenRouter failed: {e}"));
    })?;

    // Capture the HTTP status code immediately — `.text()` consumes the
    // response, so we must read it before attempting to parse the body.
    let status = response.status();

    // Read the raw body once so we can inspect it for both success and error
    // paths without consuming the response twice.
    let raw_text = response.text().await.map_err(|e| {
        return AppError::Internal(format!("failed to read response body: {e}"));
    })?;

    // If the upstream API returned a non-2xx status, parse the structured error
    // payload so the frontend sees a meaningful message rather than a generic
    // deserialization failure.
    if !status.is_success() {
        let error_response: OpenRouterErrorResponse = serde_json::from_str(&raw_text)
            .unwrap_or_else(|_| {
                // If the body is not even valid JSON, fall back to a minimal
                // error so the handler never panics.
                return OpenRouterErrorResponse {
                    error: OpenRouterError {
                        message: format!("OpenRouter returned HTTP {status} with unparseable body"),
                        code: i32::from(status.as_u16()),
                        metadata: None,
                    },
                };
            });
        eprintln!(
            "OpenRouter error ({}): {}",
            status, error_response.error.message
        );
        return Err(AppError::OpenRouter {
            status,
            message: error_response.error.message,
            code: error_response.error.code,
            metadata: error_response.error.metadata,
        });
    }

    // The upstream returned 2xx — parse the chat-completion body. An error
    // here indicates a schema mismatch between our struct and the actual API
    // response (provider API change, or the model was in streaming mode, etc.).
    let response_body: ChatCompletionResponse = serde_json::from_str(&raw_text).map_err(|e| {
        return AppError::Deserialization {
            context: "chat completion response".to_string(),
            detail: e.to_string(),
        };
    })?;
    return Ok(response_body);
}

/// Core business-logic handler for the `POST /supervisors/pick` endpoint.
///
/// # Flow
///
/// 1. Reads Qdrant / `OpenRouter` config from environment variables.
/// 2. Builds a nearest-neighbour vector query from the user's input topics and
///    biography and sends it to Qdrant (with retry).
/// 3. Collects the Qdrant payloads into a context string.
/// 4. Sends the context, along with a JSON-schema-based prompt, to the
///    `OpenRouter` chat-completions API to produce structured recommendations.
/// 5. Deserializes the LLM's response into `PickSupervisorResponse` and returns
///    it to the frontend.
///
/// # Arguments
///
/// * `payload` - Validated JSON body containing `interesting_topics` and
///   `additional_text`.
///
/// # Returns
///
/// * `Ok(Json<PickSupervisorResponse>)` — two lecturer recommendations.
/// * `Err(AppError)` — any upstream or internal failure, mapped to the
///   appropriate HTTP status code.
///
/// # Errors
///
/// Returns `AppError::OpenRouter` when the upstream API returns a non-2xx
/// status code. Returns `AppError::Deserialization` when the upstream response
/// or the LLM output does not match the expected structure. Returns
/// `AppError::EmptyChoices` if the LLM returns zero choices.
async fn pick_supervisor(
    Json(payload): Json<PickSupervisorRequest>,
) -> Result<Json<PickSupervisorResponse>, AppError> {
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
            return format!(
                "Slug: {}\nText: {}",
                item.payload.get("slug").unwrap(),
                item.payload.get("text").unwrap()
            );
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

    let response_body =
        send_openrouter_chat_completion(&openrouter_base_url, &openrouter_api_key, &query_prompt)
            .await?;

    let first_choice = response_body.choices.into_iter().next().ok_or_else(|| {
        return AppError::EmptyChoices;
    })?;
    let raw_message_content = &first_choice.message.content;

    // The LLM output should be valid JSON matching `PickSupervisorResponse`.
    // If it is not, report the raw content back so the developer can debug the
    // model's output.
    let response: PickSupervisorResponse =
        serde_json::from_str(raw_message_content).map_err(|e| {
            return AppError::Deserialization {
                context: "LLM output".to_string(),
                detail: format!("{e}. Raw content: {raw_message_content}"),
            };
        })?;
    return Ok(Json(response));
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
