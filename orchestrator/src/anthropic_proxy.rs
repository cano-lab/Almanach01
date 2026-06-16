use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

/// Configuration for the Anthropic → OpenAI proxy
#[derive(Clone)]
pub struct ProxyConfig {
    pub target_base_url: String, // e.g. "https://api.augureai.ca/v1"
    pub api_key: String,
}

/// Anthropic Messages API request format
#[derive(Debug, Deserialize)]
pub struct AnthropicRequest {
    pub model: String,
    #[serde(default)]
    pub messages: Vec<AnthropicMessage>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default)]
    pub stream: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnthropicMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
}

/// Anthropic Messages API response format
#[derive(Debug, Serialize)]
pub struct AnthropicResponse {
    pub id: String,
    pub model: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<AnthropicContentBlock>,
    pub usage: AnthropicUsage,
}

#[derive(Debug, Serialize)]
pub struct AnthropicContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Anthropic streaming event
#[derive(Debug, Serialize)]
pub struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<AnthropicDelta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<AnthropicStreamMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_block: Option<AnthropicContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<AnthropicUsage>,
}

#[derive(Debug, Serialize)]
pub struct AnthropicDelta {
    #[serde(rename = "type")]
    pub delta_type: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct AnthropicStreamMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub role: String,
    pub model: String,
    pub content: Vec<Value>,
    pub stop_reason: Option<String>,
    pub usage: AnthropicUsage,
}

/// Convert Anthropic request to OpenAI request
fn anthropic_to_openai(req: &AnthropicRequest) -> Value {
    let mut messages = Vec::new();
    
    // Add system message if present
    if let Some(system) = &req.system {
        messages.push(json!({
            "role": "system",
            "content": system
        }));
    }
    
    // Convert Anthropic messages to OpenAI format
    for msg in &req.messages {
        messages.push(json!({
            "role": &msg.role,
            "content": &msg.content
        }));
    }
    
    let mut openai_req = json!({
        "model": &req.model,
        "messages": messages,
    });
    
    if let Some(max_tokens) = req.max_tokens {
        openai_req["max_tokens"] = json!(max_tokens);
    }
    
    if let Some(temp) = req.temperature {
        openai_req["temperature"] = json!(temp);
    }
    
    if req.stream.unwrap_or(false) {
        openai_req["stream"] = json!(true);
    }
    
    openai_req
}

/// Convert OpenAI response to Anthropic response
fn openai_to_anthropic(openai_resp: &Value, model: &str) -> AnthropicResponse {
    let id = openai_resp["id"].as_str().unwrap_or("msg_").to_string();
    let choices = &openai_resp["choices"];
    let content = if let Some(choice) = choices.get(0) {
        let text = choice["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        vec![AnthropicContentBlock {
            block_type: "text".to_string(),
            text,
        }]
    } else {
        vec![]
    };
    
    let usage = if let Some(u) = openai_resp["usage"].as_object() {
        AnthropicUsage {
            input_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
        }
    } else {
        AnthropicUsage {
            input_tokens: 0,
            output_tokens: 0,
        }
    };
    
    AnthropicResponse {
        id,
        model: model.to_string(),
        response_type: "message".to_string(),
        role: "assistant".to_string(),
        content,
        usage,
    }
}

/// Handle non-streaming messages request
async fn messages_handler(
    State(config): State<Arc<ProxyConfig>>,
    Json(anthropic_req): Json<AnthropicRequest>,
) -> Result<Json<AnthropicResponse>, StatusCode> {
    let openai_req = anthropic_to_openai(&anthropic_req);
    let model = anthropic_req.model.clone();
    
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", config.target_base_url);
    
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&openai_req)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    
    let openai_json: Value = resp
        .json()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    
    let anthropic_resp = openai_to_anthropic(&openai_json, &model);
    Ok(Json(anthropic_resp))
}

/// Handle streaming messages request
async fn messages_stream_handler(
    State(config): State<Arc<ProxyConfig>>,
    Json(anthropic_req): Json<AnthropicRequest>,
) -> Response {
    let openai_req = anthropic_to_openai(&anthropic_req);
    
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", config.target_base_url);
    
    let resp = match client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&openai_req)
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => {
            return (StatusCode::BAD_GATEWAY, "Failed to connect to upstream").into_response();
        }
    };
    
    // Pass through the SSE stream directly
    // (Full translation would require parsing each SSE event and rewriting the JSON)
    let stream = resp.bytes_stream();
    let body = axum::body::Body::from_stream(stream.map(|result| {
        result.map_err(|e| axum::Error::new(e))
    }));
    
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "text/event-stream".parse().unwrap());
    headers.insert("cache-control", "no-cache".parse().unwrap());
    
    (StatusCode::OK, headers, body).into_response()
}

/// Smart handler that routes to streaming or non-streaming based on request
async fn messages_smart_handler(
    State(config): State<Arc<ProxyConfig>>,
    Json(anthropic_req): Json<AnthropicRequest>,
) -> Response {
    if anthropic_req.stream.unwrap_or(false) {
        messages_stream_handler(State(config), Json(anthropic_req)).await
    } else {
        match messages_handler(State(config), Json(anthropic_req)).await {
            Ok(resp) => resp.into_response(),
            Err(status) => status.into_response(),
        }
    }
}

/// Create the proxy router
pub fn create_proxy_router(config: ProxyConfig) -> Router {
    let state = Arc::new(config);
    
    Router::new()
        .route("/v1/messages", post(messages_smart_handler))
        .with_state(state)
}

/// Run the proxy server
pub async fn run_proxy(config: ProxyConfig, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_proxy_router(config.clone());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    println!("Anthropic proxy running on http://0.0.0.0:{}", port);
    println!("Target: {}", config.target_base_url);
    
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_anthropic_to_openai_conversion() {
        let anthropic_req = AnthropicRequest {
            model: "claude-3-haiku".to_string(),
            messages: vec![
                AnthropicMessage {
                    role: "user".to_string(),
                    content: "Hello, how are you?".to_string(),
                }
            ],
            max_tokens: Some(1024),
            temperature: Some(0.7),
            system: Some("You are helpful.".to_string()),
            stream: None,
        };
        
        let openai = anthropic_to_openai(&anthropic_req);
        
        assert_eq!(openai["model"], "claude-3-haiku");
        assert!(openai["messages"].as_array().unwrap().len() == 2); // system + user
        assert_eq!(openai["max_tokens"], 1024);
        assert_eq!(openai["temperature"], 0.7);
    }
}
