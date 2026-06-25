//! API handlers for Almanach Chat Server

use axum::{
    extract::{Extension, Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response, Sse},
    Json,
};
use axum::response::sse::Event;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::AppState;

// Re-export chat types
pub use crate::chat_db::{ChatConversation as Conversation, ChatMessage};

// === Health ===

pub async fn health(State(_state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "service": "almanach-chat"
    }))
}

// === Auth ===

#[derive(Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub role: String,
    pub username: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    let auth = state.auth.read().await;
    let token_response = auth
        .login(&req.password)
        .map_err(|e| (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let username = auth
        .validate_token(&token_response.access_token)
        .map(|c| c.sub.clone())
        .unwrap_or_else(|_| "admin".to_string());
    drop(auth);

    Ok(Json(LoginResponse {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        role: "admin".to_string(),
        username,
    }))
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub secret_word: Option<String>,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut auth = state.auth.write().await;
    let _result = auth
        .register(&req.password)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    drop(auth);

    Ok(Json(serde_json::json!({
        "success": true,
        "username": req.username,
    })))
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let auth = state.auth.read().await;
    let access_token = auth
        .refresh(&req.refresh_token)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    drop(auth);

    Ok(Json(serde_json::json!({
        "access_token": access_token,
    })))
}

pub async fn me(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    Ok(Json(serde_json::json!({
        "username": claims.sub,
        "role": claims.role.unwrap_or_else(|| "user".to_string()),
    })))
}

// === API Keys ===

#[derive(Deserialize)]
pub struct SetApiKeyRequest {
    pub provider: String,
    pub key: String,
}

#[derive(Serialize)]
pub struct ApiKeyInfo {
    pub provider: String,
    pub has_key: bool,
}

pub async fn list_api_keys(State(state): State<Arc<AppState>>) -> Json<Vec<ApiKeyInfo>> {
    let providers = ["zai", "anthropic", "openai", "kimi", "google", "augure", "fugu"];
    let keys = state.api_keys.read().await;

    Json(
        providers
            .iter()
            .map(|p| ApiKeyInfo {
                provider: p.to_string(),
                has_key: keys.contains_key(*p),
            })
            .collect(),
    )
}

pub async fn set_api_key(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Json(req): Json<SetApiKeyRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let role = claims.role.as_deref().unwrap_or_else(|| if claims.sub == "admin" { "admin" } else { "" });
    if role != "admin" && role != "teacher" && role != "student" {
        return Err((StatusCode::FORBIDDEN, "Invalid role".to_string()));
    }

    if req.key.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Key cannot be empty".to_string()));
    }

    {
        let mut keys = state.api_keys.write().await;
        keys.insert(req.provider.clone(), req.key.clone());
    }

    let keys_path = state.data_dir.join("api_keys.json");
    let keys = state.api_keys.read().await;
    let serializable: HashMap<String, String> = keys.clone();
    drop(keys);

    if let Err(e) = tokio::fs::write(&keys_path, serde_json::to_string_pretty(&serializable).unwrap_or_default()).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save key: {}", e)));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_api_key(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(provider): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let role = claims.role.as_deref().unwrap_or_else(|| if claims.sub == "admin" { "admin" } else { "" });
    if role != "admin" && role != "teacher" && role != "student" {
        return Err((StatusCode::FORBIDDEN, "Invalid role".to_string()));
    }

    {
        let mut keys = state.api_keys.write().await;
        keys.remove(&provider);
    }

    let keys_path = state.data_dir.join("api_keys.json");
    let keys = state.api_keys.read().await;
    let serializable: HashMap<String, String> = keys.clone();
    drop(keys);

    if let Err(e) = tokio::fs::write(&keys_path, serde_json::to_string_pretty(&serializable).unwrap_or_default()).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save key: {}", e)));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Return the LLM API endpoint URL for a given provider.
fn get_provider_endpoint(provider: &str) -> String {
    match provider {
        "kimi" => "https://api.kimi.com/coding/v1/messages".to_string(),
        "augure" => "https://api.augureai.ca/v1/chat/completions".to_string(),
        "openai" => "https://api.openai.com/v1/chat/completions".to_string(),
        "anthropic" => "https://api.anthropic.com/v1/messages".to_string(),
        "google" => "https://generativelanguage.googleapis.com/v1beta/models".to_string(),
        "zai" => "https://api.zai.com/v1/chat/completions".to_string(),
        _ => format!("https://api.{}.com/v1/chat/completions", provider),
    }
}

/// Return a model-appropriate max_tokens value.
fn get_model_max_tokens(provider: &str, model: &str) -> i64 {
    let model_lc = model.to_lowercase();
    match provider {
        "kimi" => 256_000,
        "anthropic" => {
            if model_lc.contains("opus") {
                16_384
            } else if model_lc.contains("sonnet") {
                8_192
            } else {
                4_096
            }
        }
        "openai" => {
            if model_lc.starts_with("gpt-4o") {
                16_384
            } else if model_lc.starts_with("gpt-4") {
                8_192
            } else {
                4_096
            }
        }
        "google" => 8_192,
        "augure" => 16_384,
        "zai" => 8_192,
        _ => 8_192,
    }
}

/// Return the models list endpoint URL for a given provider.
fn get_provider_models_endpoint(provider: &str) -> Option<String> {
    match provider {
        "openai" => Some("https://api.openai.com/v1/models".to_string()),
        "augure" => Some("https://api.augureai.ca/v1/models".to_string()),
        "zai" => Some("https://api.zai.com/v1/models".to_string()),
        "kimi" => Some("https://api.kimi.com/coding/v1/models".to_string()),
        "google" => Some("https://generativelanguage.googleapis.com/v1beta/models".to_string()),
        "anthropic" => None,
        _ => Some(format!("https://api.{}.com/v1/models", provider)),
    }
}

/// Return hardcoded fallback models for a provider.
fn get_default_models(provider: &str) -> Vec<String> {
    match provider {
        "kimi" => vec![
            "kimi-k2.7".to_string(),
            "kimi-k2.6".to_string(),
            "kimi-k2.5".to_string(),
            "kimi-k2.5-20251001".to_string(),
        ],
        "anthropic" => vec![
            "claude-sonnet-4-6".to_string(),
            "claude-opus-4-8".to_string(),
            "claude-haiku-4-5-20251001".to_string(),
        ],
        "openai" => vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
        ],
        "google" => vec![
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
        ],
        "augure" => vec![
            "augure".to_string(),
        ],
        "zai" => vec![
            "zai".to_string(),
        ],
        _ => vec!["default".to_string()],
    }
}

pub async fn list_provider_models(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Read API key
    let api_key = state
        .api_keys
        .read().await
        .get(&provider)
        .cloned()
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("No API key for provider: {}", provider) })),
            )
        })?;

    // Anthropic has no public models endpoint
    if provider == "anthropic" {
        return Ok(Json(serde_json::json!({
            "models": get_default_models("anthropic")
        })));
    }

    let client = reqwest::Client::new();

    // Try primary endpoint
    let mut endpoints: Vec<String> = Vec::new();
    if let Some(ep) = get_provider_models_endpoint(&provider) {
        endpoints.push(ep);
    }
    // Kimi fallback
    if provider == "kimi" {
        endpoints.push("https://api.kimi.com/v1/models".to_string());
    }

    let mut last_error = None;

    for endpoint in &endpoints {
        let mut req = client.get(endpoint);
        if provider == "google" {
            req = req.query(&[("key", &api_key)]);
        } else {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        match req.send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    last_error = Some(format!("HTTP {}", resp.status()));
                    continue;
                }
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let models = parse_models_response(&provider, &json);
                        if !models.is_empty() {
                            return Ok(Json(serde_json::json!({ "models": models })));
                        }
                        last_error = Some("Empty model list".to_string());
                    }
                    Err(e) => {
                        last_error = Some(format!("JSON parse error: {}", e));
                    }
                }
            }
            Err(e) => {
                last_error = Some(format!("Request failed: {}", e));
            }
        }
    }

    tracing::warn!(
        "Failed to fetch models for provider {}: {:?}. Using fallback.",
        provider,
        last_error
    );

    Ok(Json(serde_json::json!({
        "models": get_default_models(&provider)
    })))
}

fn parse_models_response(provider: &str, json: &serde_json::Value) -> Vec<String> {
    if provider == "google" {
        // Google format: { "models": [{ "name": "models/gemini-1.5-pro" }] }
        json.get("models")
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        item.get("name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.strip_prefix("models/").unwrap_or(s).to_string())
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        // OpenAI-compatible format: { "data": [{ "id": "gpt-4o" }] }
        json.get("data")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

// === Conversations ===

#[derive(Deserialize)]
pub struct CreateConversationRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
}

pub async fn list_conversations(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
) -> Result<Json<Vec<Conversation>>, (StatusCode, String)> {
    // Ensure user exists in chat_db (legacy admin tokens don't have a user record)
    let role = claims.role.as_deref().unwrap_or("admin");
    let user_role = crate::chat_db::UserRole::parse(role);
    state
        .chat_db
        .get_or_create_user_from_claims(&claims.sub, None, user_role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let convs = state
        .chat_db
        .list_conversations(&claims.sub)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(convs))
}

pub async fn create_conversation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Json(req): Json<CreateConversationRequest>,
) -> Result<Json<Conversation>, (StatusCode, String)> {
    // Ensure the user exists in chat_db (legacy admin tokens don't have a user record)
    let role = claims.role.as_deref().unwrap_or("admin");
    let user_role = crate::chat_db::UserRole::parse(role);
    state
        .chat_db
        .get_or_create_user_from_claims(&claims.sub, None, user_role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let conv = state
        .chat_db
        .create_conversation(
            &claims.sub,
            req.title.as_deref().unwrap_or("New Chat"),
            req.provider.as_deref(),
            req.model.as_deref(),
            req.temperature,
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(conv))
}

pub async fn get_conversation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
) -> Result<Json<Conversation>, (StatusCode, String)> {
    // Ensure user exists in chat_db (legacy admin tokens don't have a user record)
    let role = claims.role.as_deref().unwrap_or("admin");
    let user_role = crate::chat_db::UserRole::parse(role);
    state
        .chat_db
        .get_or_create_user_from_claims(&claims.sub, None, user_role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let conv = state
        .chat_db
        .get_conversation(&id, &claims.sub)
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(conv))
}

pub async fn delete_conversation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Ensure user exists in chat_db (legacy admin tokens don't have a user record)
    let role = claims.role.as_deref().unwrap_or("admin");
    let user_role = crate::chat_db::UserRole::parse(role);
    state
        .chat_db
        .get_or_create_user_from_claims(&claims.sub, None, user_role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state
        .chat_db
        .delete_conversation(&id, &claims.sub)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct UpdateConversationRequest {
    pub title: Option<String>,
    pub color: Option<String>,
}

pub async fn update_conversation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
    Json(req): Json<UpdateConversationRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let role = claims.role.as_deref().unwrap_or("admin");
    let user_role = crate::chat_db::UserRole::parse(role);
    state
        .chat_db
        .get_or_create_user_from_claims(&claims.sub, None, user_role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(title) = req.title {
        state
            .chat_db
            .update_conversation_title(&id, &claims.sub, &title)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    if let Some(color) = req.color {
        state
            .chat_db
            .update_conversation_color(&id, &claims.sub, &color)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(Json(serde_json::json!({"status": "updated"})))
}

/// Truncate message history to stay within a token budget.
/// Heuristic: ~4 chars per token for English. Drops oldest messages first.
fn truncate_to_budget(messages: Vec<ChatMessage>, max_tokens: usize) -> Vec<ChatMessage> {
    if messages.is_empty() {
        return messages;
    }
    let max_chars = max_tokens * 4; // rough heuristic
    let total_chars: usize = messages.iter().map(|m| m.content.len()).sum();
    if total_chars <= max_chars {
        return messages;
    }
    // Drop oldest messages until under budget, but always keep at least the last message
    let mut chars = total_chars;
    let mut skip = 0;
    while chars > max_chars && skip < messages.len().saturating_sub(1) {
        chars -= messages[skip].content.len();
        skip += 1;
    }
    messages.into_iter().skip(skip).collect()
}

// === Compact ===

pub async fn compact_conversation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
) -> Result<Json<ChatMessage>, (StatusCode, String)> {
    let role = claims.role.as_deref().unwrap_or("admin");
    let user_role = crate::chat_db::UserRole::parse(role);
    state
        .chat_db
        .get_or_create_user_from_claims(&claims.sub, None, user_role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    do_compact_conversation(&state, &id, &claims.sub).await.map(Json)
}

async fn do_compact_conversation(
    state: &AppState,
    id: &str,
    user_id: &str,
) -> Result<ChatMessage, (StatusCode, String)> {
    // Get all messages
    let history = state
        .chat_db
        .get_messages(id, user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if history.is_empty() {
        return Ok(ChatMessage {
            id: "compact-empty".to_string(),
            role: "assistant".to_string(),
            content: "Nothing to compact — this conversation is empty.".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        });
    }

    // Build summarization prompt
    let conv_text = history
        .iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n\n");

    let summary_prompt = format!(
        "Summarize the following conversation into a single concise paragraph. \
         Capture the key topics, questions, and conclusions. Keep it under 200 words. \
         This summary will replace the full conversation history.\n\n{}",
        conv_text
    );

    // Get API key
    let provider = "kimi";
    let api_key = state
        .api_keys
        .read().await
        .get(provider)
        .cloned()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("No API key for provider: {}", provider)))?;

    // Call LLM for summary
    let client = reqwest::Client::new();
    let endpoint = get_provider_endpoint(provider);
    let response = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "kimi-k2.6",
            "messages": [
                {"role": "user", "content": summary_prompt}
            ],
            "stream": false,
            "max_tokens": 512,
        }))
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("LLM request failed: {}", e)))?;

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to parse LLM response: {}", e)))?;

    let summary = response_json
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| {
            arr.iter()
                .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("")
                .into()
        })
        .or_else(|| {
            response_json
                .get("choices")
                .and_then(|c| c.as_array())
                .and_then(|c| c.first())
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "[Summary unavailable]".to_string());

    // Clear all messages
    state
        .chat_db
        .clear_messages(id, user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Store summary as a system message
    let summary_msg = state
        .chat_db
        .add_message(id, user_id, "system", &summary)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Update conversation title
    state
        .chat_db
        .update_conversation_title(id, user_id, "Compacted")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(summary_msg)
}

// === Messages ===

pub async fn get_messages(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
    Query(pagination): Query<MessagePagination>,
) -> Result<Json<Vec<ChatMessage>>, (StatusCode, String)> {
    let role = claims.role.as_deref().unwrap_or("admin");
    let user_role = crate::chat_db::UserRole::parse(role);
    state
        .chat_db
        .get_or_create_user_from_claims(&claims.sub, None, user_role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let limit = pagination.limit.unwrap_or(100).min(500);
    let offset = pagination.offset.unwrap_or(0);

    let messages = state
        .chat_db
        .get_messages_paginated(&id, &claims.sub, limit, offset)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(messages))
}

#[derive(Deserialize)]
pub struct MessagePagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub automode_role: Option<String>,
}

#[derive(Debug, Clone)]
enum SlashCommand {
    Compact,
    Temperature(f32),
    Status,
    Help,
}

fn parse_slash_command(content: &str) -> Option<SlashCommand> {
    let trimmed = content.trim();
    if trimmed.eq_ignore_ascii_case("/compact") {
        return Some(SlashCommand::Compact);
    }
    if trimmed.eq_ignore_ascii_case("/status") {
        return Some(SlashCommand::Status);
    }
    if trimmed.eq_ignore_ascii_case("/help") {
        return Some(SlashCommand::Help);
    }
    let lower = trimmed.to_lowercase();
    if let Some(rest) = lower.strip_prefix("/temp ").or_else(|| lower.strip_prefix("/temperature ")) {
        let value = rest.trim();
        if let Ok(temp) = value.parse::<f32>() {
            return Some(SlashCommand::Temperature(temp));
        }
    }
    None
}

const TEMPERATURE_HELP: &str = "Temperature controls randomness: lower values (e.g., 0.2) make responses more focused and predictable, while higher values (e.g., 1.0) make them more creative and random.";

async fn handle_slash_command(
    state: &AppState,
    conversation_id: &str,
    user_id: &str,
    cmd: SlashCommand,
) -> Result<ChatMessage, (StatusCode, String)> {
    let response = match cmd {
        SlashCommand::Compact => {
            return do_compact_conversation(state, conversation_id, user_id).await;
        }
        SlashCommand::Temperature(temp) => {
            if temp < 0.0 || temp > 2.0 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Temperature must be between 0.0 and 2.0.".to_string(),
                ));
            }
            state
                .chat_db
                .set_conversation_temperature(conversation_id, user_id, temp)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            format!("Temperature set to {:.2}.\n\n{}", temp, TEMPERATURE_HELP)
        }
        SlashCommand::Status => {
            let temp = state
                .chat_db
                .get_conversation_temperature(conversation_id, user_id)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let temp_str = temp.map(|t| format!("{:.2}", t)).unwrap_or_else(|| "default".to_string());
            format!(
                "Current settings:\n• Temperature: {}\n\n{}\n\nProvider and model are set separately via the dropdown.",
                temp_str, TEMPERATURE_HELP
            )
        }
        SlashCommand::Help => {
            "Available slash commands:\n\
             • /temp <0.0-2.0> or /temperature <0.0-2.0> — set the conversation temperature\n\
             • /status — show current AI settings\n\
             • /compact — summarize and compact the conversation history\n\
             • /help — show this help message"
                .to_string()
        }
    };

    let msg = state
        .chat_db
        .add_message(conversation_id, user_id, "system", &response)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(msg)
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<ChatMessage>, (StatusCode, String)> {
    // Ensure user exists in chat_db (legacy admin tokens don't have a user record)
    let role = claims.role.as_deref().unwrap_or("admin");
    let user_role = crate::chat_db::UserRole::parse(role);
    state
        .chat_db
        .get_or_create_user_from_claims(&claims.sub, None, user_role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Store user message (slash commands become part of history for transparency)
    let _user_msg = state
        .chat_db
        .add_message(&id, &claims.sub, "user", &req.content)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Check for slash commands
    if let Some(cmd) = parse_slash_command(&req.content) {
        return handle_slash_command(&state, &id, &claims.sub, cmd).await.map(Json);
    }

    // Get conversation history for context
    let history = state
        .chat_db
        .get_messages(&id, &claims.sub)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Apply token budget: keep system prompt + recent history
    let history = truncate_to_budget(history, 8000); // ~8K tokens budget

    // Build messages for LLM
    let mut llm_messages: Vec<serde_json::Value> = vec![];

    // Add per-conversation system prompt first, then per-user, then global active
    let conv_prompt = state.chat_db.get_conversation_system_prompt(&id).ok().flatten();
    let user_prompt = state.chat_db.get_user_system_prompt(&claims.sub).ok().flatten();
    let global_prompt = state.chat_db.get_active_system_prompt().ok().flatten();
    if let Some(prompt) = conv_prompt.or(user_prompt).or(global_prompt) {
        llm_messages.push(serde_json::json!({
            "role": "system",
            "content": prompt,
        }));
    }

    llm_messages.extend(history.iter().map(|m| {
        serde_json::json!({
            "role": m.role,
            "content": m.content,
        })
    }));

    // Determine provider and model
    let (provider, model) = {
        let (auto_provider, auto_model, method, reason) = resolve_automode_model(
            &state, &id, &claims.sub, &req.content, req.automode_role.as_deref()
        ).await?;
        if method != "manual" {
            tracing::info!("Automode routed to {}/{} via {}: {}", auto_provider, auto_model, method, reason);
            // Persist to conversation
            let _ = state.chat_db.set_conversation_provider_model(&id, &claims.sub, &auto_provider, &auto_model);
            (auto_provider, auto_model)
        } else {
            let provider = req.provider.unwrap_or_else(|| "kimi".to_string());
            let model = req.model.unwrap_or_else(|| "kimi-k2.6".to_string());
            (provider, model)
        }
    };

    // Get API key
    let api_key = state
        .api_keys
        .read().await
        .get(&provider)
        .cloned()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("No API key for provider: {}", provider)))?;

    // Call Kimi API (or other provider)
    let client = reqwest::Client::new();
    let endpoint = get_provider_endpoint(&provider);
    let max_tokens = get_model_max_tokens(&provider, &model);
    let temperature = state
        .chat_db
        .get_conversation_temperature(&id, &claims.sub)
        .ok()
        .flatten();
    let mut request_body = serde_json::json!({
        "model": model,
        "messages": llm_messages,
        "stream": false,
        "max_tokens": max_tokens,
    });
    if let Some(temp) = temperature {
        request_body["temperature"] = serde_json::json!(temp);
    }
    let response = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("LLM request failed: {}", e)))?;

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to parse LLM response: {}", e)))?;

    let assistant_content = response_json
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| {
            arr.iter()
                .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("")
                .into()
        })
        .or_else(|| {
            response_json
                .get("choices")
                .and_then(|c| c.as_array())
                .and_then(|c| c.first())
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| {
            response_json.get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("[No response]")
                .to_string()
        });

    // Store assistant message
    let assistant_msg = state
        .chat_db
        .add_message(&id, &claims.sub, "assistant", &assistant_content)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(assistant_msg))
}

// === Streaming Conversation ===

pub async fn stream_conversation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    // Auth

    let role = claims.role.as_deref().unwrap_or("admin");
    let user_role = crate::chat_db::UserRole::parse(role);
    state.chat_db.get_or_create_user_from_claims(&claims.sub, None, user_role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Save user message (slash commands become part of history for transparency)
    let _user_msg = state.chat_db.add_message(&id, &claims.sub, "user", &req.content)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Check for slash commands
    if let Some(cmd) = parse_slash_command(&req.content) {
        match handle_slash_command(&state, &id, &claims.sub, cmd).await {
            Ok(msg) => {
                let event_data = serde_json::json!({ "text": msg.content }).to_string();
                let events = vec![
                    Ok::<_, axum::Error>(Event::default().data(event_data)),
                    Ok::<_, axum::Error>(Event::default().data("[DONE]")),
                ];
                return Ok(Sse::new(futures::stream::iter(events)).into_response());
            }
            Err(e) => return Err(e),
        }
    }

    // Build context
    let history = state.chat_db.get_messages(&id, &claims.sub)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let history = truncate_to_budget(history, 8000);

    let mut llm_messages: Vec<serde_json::Value> = vec![];

    let conv_prompt = state.chat_db.get_conversation_system_prompt(&id).ok().flatten();
    let user_prompt = state.chat_db.get_user_system_prompt(&claims.sub).ok().flatten();
    let global_prompt = state.chat_db.get_active_system_prompt().ok().flatten();
    if let Some(prompt) = conv_prompt.or(user_prompt).or(global_prompt) {
        llm_messages.push(serde_json::json!({ "role": "system", "content": prompt }));
    }

    llm_messages.extend(history.iter().map(|m| {
        serde_json::json!({ "role": m.role, "content": m.content })
    }));

    // Determine provider and model: automode > request overrides > stored > defaults
    let (provider, model) = {
        let (auto_provider, auto_model, method, reason) = resolve_automode_model(
            &state, &id, &claims.sub, &req.content, req.automode_role.as_deref()
        ).await?;
        if method != "manual" {
            tracing::info!("Automode routed to {}/{} via {}: {}", auto_provider, auto_model, method, reason);
            // Persist to conversation
            let _ = state.chat_db.set_conversation_provider_model(&id, &claims.sub, &auto_provider, &auto_model);
            (auto_provider, auto_model)
        } else if req.provider.is_some() || req.model.is_some() {
            let provider = req.provider.unwrap_or_else(|| "kimi".to_string());
            let model = req.model.unwrap_or_else(|| "kimi-k2.7".to_string());
            // Persist to conversation
            let _ = state.chat_db.set_conversation_provider_model(&id, &claims.sub, &provider, &model);
            (provider, model)
        } else {
            match state.chat_db.get_conversation_provider_model(&id, &claims.sub).ok().flatten() {
                Some((p, m)) => (p, m),
                None => ("kimi".to_string(), "kimi-k2.7".to_string()),
            }
        }
    };

    let api_key = state.api_keys.read().await.get(&provider)
        .cloned()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("No API key for provider: {}", provider)))?;

    let temperature = state
        .chat_db
        .get_conversation_temperature(&id, &claims.sub)
        .ok()
        .flatten();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<Event, axum::Error>>();

    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let endpoint = get_provider_endpoint(&provider);
        tracing::info!("Streaming chat request: provider={}, model={}, endpoint={}", provider, model, endpoint);

        // Create an empty assistant message at the start of the stream.
        let assistant_msg = match state.chat_db.add_message(&id, &claims.sub, "assistant", "") {
            Ok(msg) => msg,
            Err(e) => {
                tracing::error!("Failed to create assistant message: {}", e);
                let _ = tx.send(Ok(Event::default().data(format!("{{\"error\": \"Failed to persist message: {}\"}}", e))));
                return;
            }
        };
        let assistant_msg_id = assistant_msg.id;

        let max_tokens = get_model_max_tokens(&provider, &model);
        let mut request_body = serde_json::json!({
            "model": model,
            "messages": llm_messages,
            "stream": true,
            "max_tokens": max_tokens,
        });
        if let Some(temp) = temperature {
            request_body["temperature"] = serde_json::json!(temp);
        }
        let response = client.post(&endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await;

        let mut response = match response {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("LLM request failed: {}", e);
                let err_text = format!("Error: {}", e);
                let _ = tx.send(Ok(Event::default().data(format!("{{\"error\": \"{}\"}}", e))));
                let _ = state.chat_db.update_message(&assistant_msg_id, &err_text);
                return;
            }
        };

        // Check HTTP status
        tracing::info!("LLM response status: {}", response.status());
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!("LLM error response: status={}, body={}", status, body);
            let err = format!("LLM returned {}: {}", status, body);
            let _ = tx.send(Ok(Event::default().data(format!("{{\"error\": \"{}\"}}", err))));
            let _ = state.chat_db.update_message(&assistant_msg_id, &err);
            return;
        }

        let mut full_text = String::new();
        let mut buffer = String::new();
        let mut chunk_count = 0;
        let mut last_persisted_len = 0;

        loop {
            match response.chunk().await {
                Ok(Some(chunk)) => {
                    chunk_count += 1;
                    let chunk_str = String::from_utf8_lossy(&chunk);
                    if chunk_count <= 5 {
                        tracing::info!("LLM chunk {}: {}", chunk_count, chunk_str);
                    }
                    buffer.push_str(&chunk_str);

                    // Process complete lines
                    while let Some(pos) = buffer.find('\n') {
                        let line = buffer[..pos].trim_end_matches('\r').to_string();
                        buffer = buffer[pos + 1..].to_string();

                        if let Some(data) = line.strip_prefix("data: ").or_else(|| line.strip_prefix("data:")) {
                            if data == "[DONE]" {
                                continue;
                            }

                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(content) = extract_stream_delta(&json) {
                                    full_text.push_str(&content);
                                }
                            }

                            // Forward the SSE event
                            let _ = tx.send(Ok(Event::default().data(data)));
                        }
                    }

                    // Persist periodically when content length changes meaningfully
                    if full_text.len() > last_persisted_len {
                        let _ = state.chat_db.update_message(&assistant_msg_id, &full_text);
                        last_persisted_len = full_text.len();
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    let err = format!("Stream error: {}", e);
                    let _ = tx.send(Ok(Event::default().data(format!("{{\"error\": \"{}\"}}", err))));
                    let _ = state.chat_db.update_message(&assistant_msg_id, &err);
                    return;
                }
            }
        }

        // Save the final assistant message
        tracing::info!("LLM stream complete: chunks={}, full_text_len={}", chunk_count, full_text.len());
        if !full_text.is_empty() {
            let _ = state.chat_db.update_message(&assistant_msg_id, &full_text);
        }
    });

    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
    Ok(Sse::new(stream).into_response())
}

fn extract_stream_delta(json: &serde_json::Value) -> Option<String> {
    // OpenAI standard format: choices[0].delta.content (string)
    if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
        if let Some(choice) = choices.first() {
            if let Some(delta) = choice.get("delta") {
                // Standard string content
                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                    return Some(content.to_string());
                }
                // Kimi coding format: array of text parts
                if let Some(content_array) = delta.get("content").and_then(|c| c.as_array()) {
                    let text: String = content_array
                        .iter()
                        .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                        .collect::<Vec<_>>()
                        .join("");
                    return Some(text);
                }
                // Anthropic-style: delta.text
                if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                    return Some(text.to_string());
                }
            }
        }
    }

    // Kimi text_delta format: { type: "content_block_delta", delta: { type: "text_delta", text: "..." } }
    if let Some(delta) = json.get("delta") {
        if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
            return Some(text.to_string());
        }
    }

    // Direct content array (Kimi non-streaming style, sometimes seen in streaming)
    if let Some(content_array) = json.get("content").and_then(|c| c.as_array()) {
        let text: String = content_array
            .iter()
            .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join("");
        if !text.is_empty() {
            return Some(text);
        }
    }

    // Direct text field
    if let Some(text) = json.get("text").and_then(|t| t.as_str()) {
        return Some(text.to_string());
    }

    None
}

// === Streaming Chat ===

#[derive(Deserialize)]
pub struct ChatStreamRequest {
    pub conversation_id: String,
    pub content: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub automode_role: Option<String>,
}

pub async fn chat_stream(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Json(req): Json<ChatStreamRequest>,
) -> Result<Json<ChatMessage>, (StatusCode, String)> {
    // For now, non-streaming. Store user message and return assistant response.
    let _user_msg = state
        .chat_db
        .add_message(&req.conversation_id, &claims.sub, "user", &req.content)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Check for slash commands
    if let Some(cmd) = parse_slash_command(&req.content) {
        return handle_slash_command(&state, &req.conversation_id, &claims.sub, cmd)
            .await
            .map(Json);
    }

    let history = state
        .chat_db
        .get_messages(&req.conversation_id, &claims.sub)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let llm_messages: Vec<serde_json::Value> = history
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content,
            })
        })
        .collect();

    let provider = req.provider.unwrap_or_else(|| "kimi".to_string());
    let model = req.model.unwrap_or_else(|| "kimi-k2.6".to_string());

    // Get API key
    let api_key = state
        .api_keys
        .read().await
        .get(&provider)
        .cloned()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("No API key for provider: {}", provider)))?;

    let client = reqwest::Client::new();
    let endpoint = get_provider_endpoint(&provider);
    let max_tokens = get_model_max_tokens(&provider, &model);
    let temperature = state
        .chat_db
        .get_conversation_temperature(&req.conversation_id, &claims.sub)
        .ok()
        .flatten();
    let mut request_body = serde_json::json!({
        "model": model,
        "messages": llm_messages,
        "stream": false,
        "max_tokens": max_tokens,
    });
    if let Some(temp) = temperature {
        request_body["temperature"] = serde_json::json!(temp);
    }
    let response = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("LLM request failed: {}", e)))?;

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to parse LLM response: {}", e)))?;

    let content = response_json
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| {
            arr.iter()
                .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("")
                .into()
        })
        .or_else(|| {
            response_json
                .get("choices")
                .and_then(|c| c.as_array())
                .and_then(|c| c.first())
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| {
            response_json.get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("[No response]")
                .to_string()
        });

    let assistant_msg = state
        .chat_db
        .add_message(&req.conversation_id, &claims.sub, "assistant", &content)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(assistant_msg))
}

// === WebSocket Chat ===

pub async fn chat_websocket(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    ws: WebSocketUpgrade,
) -> Result<Response, (StatusCode, String)> {
    let token = params
        .get("token")
        .ok_or((StatusCode::UNAUTHORIZED, "Missing token".to_string()))?;

    let auth = state.auth.read().await;
    let claims = auth
        .validate_token(token)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    drop(auth);

    let user_id = claims.sub.clone();
    let state = state.clone();

    Ok(ws.on_upgrade(move |socket| handle_chat_ws(socket, state, user_id)))
}

async fn handle_chat_ws(
    mut socket: axum::extract::ws::WebSocket,
    state: Arc<AppState>,
    user_id: String,
) {
    use axum::extract::ws::Message;

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(req) = serde_json::from_str::<ChatStreamRequest>(&text) {
                    // Store user message
                    let _ = state.chat_db.add_message(
                        &req.conversation_id,
                        &user_id,
                        "user",
                        &req.content,
                    );

                    // Check for slash commands
                    if let Some(cmd) = parse_slash_command(&req.content) {
                        let reply = handle_slash_command(
                            &state,
                            &req.conversation_id,
                            &user_id,
                            cmd,
                        )
                        .await
                        .map(|msg| msg.content)
                        .unwrap_or_else(|(_, e)| e);
                        let _ = socket.send(Message::Text(reply)).await;
                        continue;
                    }

                    // Get history
                    let history = match state.chat_db.get_messages(&req.conversation_id, &user_id) {
                        Ok(h) => h,
                        Err(_) => continue,
                    };

                    let llm_messages: Vec<serde_json::Value> = history
                        .iter()
                        .map(|m| {
                            serde_json::json!({
                                "role": m.role,
                                "content": m.content,
                            })
                        })
                        .collect();

                    let provider = req.provider.unwrap_or_else(|| "kimi".to_string());
                    let model = req.model.unwrap_or_else(|| "kimi-k2.6".to_string());

                    let api_key = match state.api_keys.read().await.get(&provider) {
                        Some(k) => k.clone(),
                        None => continue,
                    };

                    // Non-streaming for WebSocket simplicity
                    let client = reqwest::Client::new();
                    let endpoint = get_provider_endpoint(&provider);
                    let max_tokens = get_model_max_tokens(&provider, &model);
                    let temperature = state
                        .chat_db
                        .get_conversation_temperature(&req.conversation_id, &user_id)
                        .ok()
                        .flatten();
                    let mut request_body = serde_json::json!({
                        "model": model,
                        "messages": llm_messages,
                        "stream": false,
                        "max_tokens": max_tokens,
                    });
                    if let Some(temp) = temperature {
                        request_body["temperature"] = serde_json::json!(temp);
                    }
                    if let Ok(response) = client
                        .post(&endpoint)
                        .header("Authorization", format!("Bearer {}", api_key))
                        .header("Content-Type", "application/json")
                        .json(&request_body)
                        .send()
                        .await
                    {
                        if let Ok(response_json) = response.json::<serde_json::Value>().await {
                            let content = response_json
                                .get("choices")
                                .and_then(|c| c.as_array())
                                .and_then(|c| c.first())
                                .and_then(|c| c.get("message"))
                                .and_then(|m| m.get("content"))
                                .and_then(|c| c.as_str())
                                .unwrap_or("[No response]");

                            // Store assistant message
                            let _ = state.chat_db.add_message(
                                &req.conversation_id,
                                &user_id,
                                "assistant",
                                content,
                            );

                            let _ = socket
                                .send(Message::Text(
                                    serde_json::json!({
                                        "role": "assistant",
                                        "content": content,
                                        "conversation_id": req.conversation_id,
                                    })
                                    .to_string(),
                                ))
                                .await;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }
}

// === Admin ===

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {

    if claims.role.as_deref() != Some("admin") && claims.role.as_deref() != Some("teacher") && claims.sub != "admin" {
        return Err((StatusCode::FORBIDDEN, "Admin access required".to_string()));
    }

    let users = state.chat_db.list_all_users()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let json_users: Vec<serde_json::Value> = users.into_iter().map(|u| {
        serde_json::json!({
            "id": u.id,
            "username": u.username,
            "display_name": u.display_name,
            "role": u.role.as_str(),
            "approval_status": u.approval_status.as_str(),
            "created_at": u.created_at,
        })
    }).collect();

    Ok(Json(json_users))
}

#[derive(Deserialize)]
pub struct ApproveUserRequest {
    pub username: String,
}

pub async fn approve_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Json(req): Json<ApproveUserRequest>,
) -> Result<StatusCode, (StatusCode, String)> {

    if claims.role.as_deref() != Some("admin") && claims.role.as_deref() != Some("teacher") && claims.sub != "admin" {
        return Err((StatusCode::FORBIDDEN, "Admin access required".to_string()));
    }

    // Find user by username
    let user = state.chat_db.get_user_by_username(&req.username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    state.chat_db.update_user_status(&user.id, crate::chat_db::ApprovalStatus::Approved)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {

    if claims.role.as_deref() != Some("admin") && claims.sub != "admin" {
        return Err((StatusCode::FORBIDDEN, "Admin access required".to_string()));
    }

    state.chat_db.delete_user(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct ConversationSettingsRequest {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
}

pub async fn set_conversation_settings(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
    Json(req): Json<ConversationSettingsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let role = claims.role.as_deref().unwrap_or("admin");
    let user_role = crate::chat_db::UserRole::parse(role);
    state
        .chat_db
        .get_or_create_user_from_claims(&claims.sub, None, user_role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let (Some(provider), Some(model)) = (req.provider.as_ref(), req.model.as_ref()) {
        state
            .chat_db
            .set_conversation_provider_model(&id, &claims.sub, provider, model)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    if let Some(temp) = req.temperature {
        if temp < 0.0 || temp > 2.0 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Temperature must be between 0.0 and 2.0.".to_string(),
            ));
        }
        state
            .chat_db
            .set_conversation_temperature(&id, &claims.sub, temp)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(Json(serde_json::json!({"status": "updated"})))
}

pub async fn set_conversation_system_prompt(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {

    let prompt = req.get("prompt").and_then(|v| v.as_str());
    state.chat_db.set_conversation_system_prompt(&id, prompt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}

pub async fn list_system_prompt_templates(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    let prompts = state.chat_db.list_system_prompts()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let json_prompts = prompts.into_iter().map(|p| {
        serde_json::json!({
            "id": p.id,
            "name": p.name,
            "content": p.content,
            "is_active": p.is_active,
        })
    }).collect();

    Ok(Json(json_prompts))
}

pub async fn list_courses(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    let courses = state.chat_db.list_courses()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Return the catalog as the legacy bare array, but include enough nested
    // module/lesson metadata for the admin UI to show useful counts without a
    // second request per course.
    let mut json_courses = Vec::new();
    for c in courses {
        let mut modules = state.chat_db.get_course_modules(&c.id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let mut lesson_count = 0usize;
        for module in &mut modules {
            let lessons = state.chat_db.get_module_lessons(&module.id)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            lesson_count += lessons.len();
            module.lessons = lessons;
        }

        let module_count = modules.len();
        json_courses.push(serde_json::json!({
            "id": c.id,
            "title": c.title,
            "title_en": c.title_en,
            "description": c.description,
            "grade": c.grade,
            "language": c.language,
            "credit_hours": c.credit_hours,
            "module_count": module_count,
            "lesson_count": lesson_count,
            "modules": modules.into_iter().map(|m| serde_json::json!({
                "id": m.id,
                "title": m.title,
                "title_en": m.title_en,
                "description": m.description,
                "order": m.order_index,
                "estimated_hours": m.estimated_hours,
                "lessons": m.lessons.into_iter().map(|l| serde_json::json!({
                    "id": l.id,
                    "title": l.title,
                    "title_en": l.title_en,
                    "description": l.description,
                    "topics": l.topics,
                    "objectives": l.objectives,
                    "estimated_minutes": l.estimated_minutes,
                    "order": l.order_index,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        }));
    }

    Ok(Json(json_courses))
}

pub async fn get_course(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let course = match state.chat_db.get_course(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))? {
        Some(c) => c,
        None => return Err((StatusCode::NOT_FOUND, "Course not found".to_string())),
    };

    let mut modules = state.chat_db.get_course_modules(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for module in &mut modules {
        let lessons = state.chat_db.get_module_lessons(&module.id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        module.lessons = lessons;
    }

    Ok(Json(serde_json::json!({
        "id": course.id,
        "title": course.title,
        "title_en": course.title_en,
        "description": course.description,
        "grade": course.grade,
        "language": course.language,
        "credit_hours": course.credit_hours,
        "modules": modules.into_iter().map(|m| serde_json::json!({
            "id": m.id,
            "title": m.title,
            "title_en": m.title_en,
            "description": m.description,
            "order": m.order_index,
            "estimated_hours": m.estimated_hours,
            "lessons": m.lessons.into_iter().map(|l| serde_json::json!({
                "id": l.id,
                "title": l.title,
                "title_en": l.title_en,
                "description": l.description,
                "topics": l.topics,
                "objectives": l.objectives,
                "estimated_minutes": l.estimated_minutes,
                "order": l.order_index,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })))
}

pub async fn enroll_course(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {

    let enrollment = state.chat_db.enroll_user(&claims.sub, &id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "id": enrollment.id,
        "course_id": enrollment.course_id,
        "status": enrollment.status,
    })))
}

pub async fn list_enrollments(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {

    let enrollments = state.chat_db.list_enrollments(&claims.sub)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let json = enrollments.into_iter().map(|e| {
        serde_json::json!({
            "id": e.id,
            "course_id": e.course_id,
            "status": e.status,
            "enrolled_at": e.enrolled_at,
        })
    }).collect();

    Ok(Json(json))
}

pub async fn start_lesson(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {

    let lesson = match state.chat_db.get_lesson(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))? {
        Some(l) => l,
        None => return Err((StatusCode::NOT_FOUND, "Lesson not found".to_string())),
    };

    // Check if conversation already exists for this lesson
    let existing = state.chat_db.get_conversation_by_lesson(&claims.sub, &id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(conv) = existing {
        return Ok(Json(serde_json::json!({
            "conversation_id": conv.id,
            "lesson_id": lesson.id,
            "title": lesson.title,
        })));
    }

    let title = format!("{} — {}", lesson.title, lesson.title_en.clone().unwrap_or_default());
    let system_prompt = if lesson.system_prompt.is_empty() {
        "You are a helpful math tutor for Grade 9 students.".to_string()
    } else {
        lesson.system_prompt.clone()
    };

    let conv = state.chat_db.create_lesson_conversation(&claims.sub, &id, &title, &system_prompt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "conversation_id": conv.id,
        "lesson_id": lesson.id,
        "title": lesson.title,
    })))
}

// ─── Additional Course Endpoints ───────────────────────────────────────────

#[derive(Deserialize)]
pub struct UpdateProgressRequest {
    pub enrollment_id: String,
    pub lesson_id: String,
    pub status: String, // "not_started", "in_progress", "completed"
}

pub async fn get_module_lessons(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let lessons = state.chat_db.get_module_lessons(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "lessons": lessons.into_iter().map(|l| serde_json::json!({
            "id": l.id,
            "title": l.title,
            "title_en": l.title_en,
            "description": l.description,
            "estimated_minutes": l.estimated_minutes,
            "order": l.order_index,
        })).collect::<Vec<_>>(),
    })))
}

pub async fn get_lesson_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let lesson = match state.chat_db.get_lesson(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))? {
        Some(l) => l,
        None => return Err((StatusCode::NOT_FOUND, "Lesson not found".to_string())),
    };

    Ok(Json(serde_json::json!({
        "id": lesson.id,
        "title": lesson.title,
        "title_en": lesson.title_en,
        "description": lesson.description,
        "topics": lesson.topics,
        "objectives": lesson.objectives,
        "estimated_minutes": lesson.estimated_minutes,
        "keywords": lesson.keywords,
        "order": lesson.order_index,
    })))
}

pub async fn update_progress(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<crate::auth::Claims>,
    Json(req): Json<UpdateProgressRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {

    let valid_status = matches!(req.status.as_str(), "not_started" | "in_progress" | "completed");
    if !valid_status {
        return Err((StatusCode::BAD_REQUEST, "Invalid status. Use: not_started, in_progress, completed".to_string()));
    }

    let progress = state.chat_db.update_course_lesson_progress(&req.enrollment_id, &req.lesson_id, &req.status)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "id": progress.id,
        "enrollment_id": progress.enrollment_id,
        "lesson_id": progress.lesson_id,
        "status": progress.status,
        "started_at": progress.started_at,
        "completed_at": progress.completed_at,
    })))
}

pub async fn get_enrollment_progress(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {

    let progress = state.chat_db.get_enrollment_progress(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "enrollment_id": id,
        "progress": progress.into_iter().map(|p| serde_json::json!({
            "id": p.id,
            "lesson_id": p.lesson_id,
            "status": p.status,
            "started_at": p.started_at,
            "completed_at": p.completed_at,
            "last_activity_at": p.last_activity_at,
        })).collect::<Vec<_>>(),
    })))
}

pub async fn get_lesson_chat(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {

    let conv = state.chat_db.get_conversation_by_lesson(&claims.sub, &id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match conv {
        Some(c) => Ok(Json(serde_json::json!({
            "has_chat": true,
            "conversation_id": c.id,
            "title": c.title,
        }))),
        None => Ok(Json(serde_json::json!({
            "has_chat": false,
            "conversation_id": null,
        }))),
    }
}

// === Terminal Agent API ───────────────────────────────────────────────────

pub async fn terminal_mount(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MountRequest>,
) -> Result<Json<crate::terminal_agent::TerminalSession>, (StatusCode, String)> {
    match state.terminal.mount(&req.path).await {
        Ok(session) => Ok(Json(session)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn terminal_list_sessions(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let sessions = state.terminal.list_sessions().await;
    Json(serde_json::json!({ "sessions": sessions }))
}

pub async fn terminal_unmount(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    match state.terminal.unmount(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn terminal_exec(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ExecRequest>,
) -> Result<Json<crate::terminal_agent::ExecResult>, (StatusCode, String)> {
    match state.terminal.exec(&id, &req.command).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn terminal_read(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<FilePathRequest>,
) -> Result<Json<crate::terminal_agent::FileContent>, (StatusCode, String)> {
    match state.terminal.read_file(&id, &req.path).await {
        Ok(content) => Ok(Json(content)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn terminal_write(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<WriteFileRequest>,
) -> Result<Json<crate::terminal_agent::WriteResult>, (StatusCode, String)> {
    match state.terminal.write_file(&id, &req.path, &req.content).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn terminal_ls(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<FilePathRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.terminal.list_dir(&id, &req.path).await {
        Ok(entries) => Ok(Json(serde_json::json!({ "entries": entries }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

#[derive(Debug, Deserialize)]
pub struct MountRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct ExecRequest {
    pub command: String,
}

#[derive(Debug, Deserialize)]
pub struct FilePathRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct WriteFileRequest {
    pub path: String,
    pub content: String,
}

// ─── Automode ───────────────────────────────────────────────────────────────

use crate::automode::{self, AutomodeConfig, DecisionRecord, RouteRequest};
use tokio::sync::RwLock;

#[derive(Debug, Deserialize)]
pub struct UpdateAutomodeConfigRequest {
    pub enabled: Option<bool>,
    pub default_role: Option<String>,
}

pub async fn get_automode_config(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
) -> Result<Json<AutomodeConfig>, (StatusCode, String)> {
    let role = claims.role.as_deref().unwrap_or_else(|| if claims.sub == "admin" { "admin" } else { "" });
    if role != "admin" && role != "teacher" {
        return Err((StatusCode::FORBIDDEN, "Admin/teacher only".to_string()));
    }
    let config = state.automode_config.read().await;
    Ok(Json(config.clone()))
}

pub async fn update_automode_config(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Json(req): Json<UpdateAutomodeConfigRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let role = claims.role.as_deref().unwrap_or_else(|| if claims.sub == "admin" { "admin" } else { "" });
    if role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Admin only".to_string()));
    }
    
    let mut config = state.automode_config.write().await;
    if let Some(enabled) = req.enabled {
        config.enabled = enabled;
    }
    if let Some(default_role) = req.default_role {
        config.default_role = default_role;
    }
    
    if let Err(e) = automode::save_automode_config(&state.data_dir, &*config) {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }
    
    let router = if config.enabled {
        Some(automode::AutomodeRouter::new(
            config.clone(),
            Arc::new(RwLock::new({
                let keys = state.api_keys.read().await;
                keys.clone()
            })),
        ))
    } else {
        None
    };
    
    let mut router_lock = state.automode_router.write().await;
    *router_lock = router;
    
    Ok(Json(serde_json::json!({
        "status": "ok",
        "enabled": config.enabled,
        "default_role": config.default_role,
    })))
}

pub async fn list_automode_roles(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let role = claims.role.as_deref().unwrap_or_else(|| if claims.sub == "admin" { "admin" } else { "" });
    if role != "admin" && role != "teacher" && role != "student" {
        return Err((StatusCode::FORBIDDEN, "Invalid role".to_string()));
    }
    let config = state.automode_config.read().await;
    Ok(Json(serde_json::json!({
        "roles": config.roles.iter().map(|r| serde_json::json!({
            "id": r.id,
            "name": r.name,
            "description": r.description,
            "requires_vision": r.requires_vision,
            "required_languages": r.required_languages,
            "default_sensitivity": r.default_sensitivity,
            "preferred_strengths": r.preferred_strengths,
        })).collect::<Vec<_>>(),
        "enabled": config.enabled,
        "default_role": config.default_role,
    })))
}

/// Student-facing model cards: all authenticated users can see model specs
pub async fn list_model_cards(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<crate::auth::Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let config = state.automode_config.read().await;
    
    let models = config.models.iter().filter(|m| m.enabled).map(|m| serde_json::json!({
        "id": m.id,
        "friendly_name": m.friendly_name,
        "description": m.description,
        "provider": m.provider,
        "model_string": m.model_string,
        "context_window": m.context_window,
        "max_output_tokens": m.max_output_tokens,
        "vision": m.vision,
        "languages": m.languages,
        "tool_use": m.tool_use,
        "cost_tier": m.cost_tier,
        "latency_tier": m.latency_tier,
        "strengths": m.strengths,
        "is_orchestrator": m.is_orchestrator,
        "orchestrator_type": m.orchestrator_type,
    })).collect::<Vec<_>>();
    
    let roles = config.roles.iter().map(|r| serde_json::json!({
        "id": r.id,
        "name": r.name,
        "description": r.description,
        "temperature": r.temperature,
        "default_sensitivity": r.default_sensitivity,
        "recommended_models": r.recommended_models,
        "preferred_strengths": r.preferred_strengths,
    })).collect::<Vec<_>>();
    
    Ok(Json(serde_json::json!({
        "automode_enabled": config.enabled,
        "default_role": config.default_role,
        "models": models,
        "roles": roles,
    })))
}

#[derive(Debug, Deserialize)]
pub struct RouteAutomodeRequest {
    pub role_id: String,
    pub payload_summary: String,
    pub sensitivity: Option<String>,
    pub grade_level: Option<String>,
}

pub async fn route_automode(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<crate::auth::Claims>,
    Json(req): Json<RouteAutomodeRequest>,
) -> Result<Json<DecisionRecord>, (StatusCode, String)> {
    let router = state.automode_router.read().await;
    let router = match router.as_ref() {
        Some(r) => r,
        None => return Err((StatusCode::SERVICE_UNAVAILABLE, "Automode not enabled".to_string())),
    };
    
    let route_req = RouteRequest {
        role_id: req.role_id,
        payload_summary: req.payload_summary,
        sensitivity: req.sensitivity,
        grade_level: req.grade_level,
    };
    
    let decision = router.route(route_req).await;
    Ok(Json(decision))
}

/// Helper: if automode is enabled, route the request to select the best model.
/// Returns (provider, model, method, reason) — method is "manual" if automode is off.
async fn resolve_automode_model(
    state: &Arc<AppState>,
    conversation_id: &str,
    user_id: &str,
    content: &str,
    requested_role: Option<&str>,
) -> Result<(String, String, String, String), (StatusCode, String)> {
    let config = state.automode_config.read().await;
    if !config.enabled {
        return Ok(("".to_string(), "".to_string(), "manual".to_string(), "Automode disabled".to_string()));
    }
    
    // Get the role: requested > conversation stored > default
    let role_id = if let Some(role) = requested_role {
        role.to_string()
    } else if let Ok(Some(stored)) = state.chat_db.get_conversation_automode_role(conversation_id, user_id) {
        stored
    } else {
        config.default_role.clone()
    };
    
    // If requested role differs from stored, update it
    if requested_role.is_some() {
        let _ = state.chat_db.set_conversation_automode_role(conversation_id, user_id, &role_id);
    }
    
    let router = state.automode_router.read().await;
    let router = match router.as_ref() {
        Some(r) => r,
        None => return Ok(("".to_string(), "".to_string(), "manual".to_string(), "Router not initialized".to_string())),
    };
    
    let route_req = RouteRequest {
        role_id: role_id.clone(),
        payload_summary: content.to_string(),
        sensitivity: None,
        grade_level: None,
    };
    
    let decision = router.route(route_req).await;
    
    if let Some(model) = decision.chosen_model {
        let provider = decision.chosen_orchestrator.unwrap_or_else(|| "kimi".to_string());
        return Ok((provider, model, decision.method, decision.reason));
    }
    
    // Escalation or no model found — fall back to manual
    Ok(("kimi".to_string(), "kimi-k2.7".to_string(), "escalation".to_string(), decision.reason))
}

#[derive(Debug, Deserialize)]
pub struct SetAutomodeRoleRequest {
    pub role_id: String,
}

pub async fn set_conversation_automode_role(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Path(id): Path<String>,
    Json(req): Json<SetAutomodeRoleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let role = claims.role.as_deref().unwrap_or_else(|| if claims.sub == "admin" { "admin" } else { "" });
    let user_role = crate::chat_db::UserRole::parse(role);
    state
        .chat_db
        .get_or_create_user_from_claims(&claims.sub, None, user_role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state
        .chat_db
        .set_conversation_automode_role(&id, &claims.sub, &req.role_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "status": "ok",
        "automode_role": req.role_id,
    })))
}

pub async fn get_automode_status(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let config = state.automode_config.read().await;
    Json(serde_json::json!({
        "enabled": config.enabled,
        "default_role": config.default_role,
        "roles": config.roles.iter().map(|r| serde_json::json!({
            "id": r.id,
            "name": r.name,
            "description": r.description,
            "requires_vision": r.requires_vision,
            "required_languages": r.required_languages,
            "default_sensitivity": r.default_sensitivity,
            "preferred_strengths": r.preferred_strengths,
        })).collect::<Vec<_>>(),
    }))
}
