use std::collections::HashMap;
use std::sync::Arc;

mod anthropic_proxy;
mod api;
mod auth;
mod chat_db;
mod config;
mod courses;
mod course_api;
mod terminal_agent;

use axum::http::{header, HeaderName, Method};
use axum::{
    routing::{get, post, delete, patch, put},
    Router,
};
use tokio::sync::RwLock;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::ServeDir;

use crate::auth::AuthManager;
use crate::chat_db::ChatDb;

pub struct AppState {
    pub config: config::Config,
    pub api_keys: RwLock<HashMap<String, String>>,
    pub data_dir: std::path::PathBuf,
    pub auth: RwLock<AuthManager>,
    pub chat_db: Arc<ChatDb>,
    pub terminal: terminal_agent::TerminalAgent,
}

fn load_api_keys(data_dir: &std::path::Path) -> HashMap<String, String> {
    // Keep legacy data/api_keys.json support, then let environment variables
    // override it. The UI intentionally treats key management as read-only and
    // points operators to these env vars.
    let mut keys: HashMap<String, String> = HashMap::new();

    let keys_path = data_dir.join("api_keys.json");
    if keys_path.exists() {
        if let Ok(contents) = std::fs::read_to_string(&keys_path) {
            if let Ok(file_keys) = serde_json::from_str::<HashMap<String, String>>(&contents) {
                keys.extend(file_keys);
            }
        }
    }

    for (provider, env_name) in [
        ("augure", "AUGURE_API_KEY"),
        ("anthropic", "ANTHROPIC_API_KEY"),
        ("openai", "OPENAI_API_KEY"),
        ("kimi", "KIMI_API_KEY"),
        ("google", "GOOGLE_API_KEY"),
        ("zai", "ZAI_API_KEY"),
    ] {
        if let Ok(value) = std::env::var(env_name) {
            if !value.trim().is_empty() {
                keys.insert(provider.to_string(), value);
            }
        }
    }

    keys
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--set-password".to_string()) {
        let data_dir = std::path::PathBuf::from("./data");
        auth::cli_set_password(&data_dir)?;
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    let config = config::load()?;
    let data_dir = std::path::PathBuf::from("./data");
    std::fs::create_dir_all(&data_dir).ok();
    tracing::info!("Loaded config: {:?}", config);

    let auth_manager = AuthManager::new(&data_dir)?;
    if !auth_manager.has_admin() {
        tracing::warn!("⚠️  No admin password set. Use --set-password to set one.");
    }

    let api_keys = load_api_keys(&data_dir);
    let chat_db = Arc::new(ChatDb::open(&data_dir.join("chat.db"))?);

    let state = Arc::new(AppState {
        config: config.clone(),
        api_keys: RwLock::new(api_keys),
        data_dir: data_dir.clone(),
        auth: RwLock::new(auth_manager),
        chat_db,
        terminal: terminal_agent::TerminalAgent::new(),
    });

    let static_dir = config
        .static_dir
        .clone()
        .unwrap_or_else(|| "./static-site".to_string());

    let app = Router::new()
        .route("/health", get(api::health))
        .route("/api/me", get(auth::me))
        .route("/auth/me", get(auth::me))
        .route("/auth/login", post(api::login))
        .route("/auth/user/login", post(auth::user_login))
        .route("/auth/register", post(api::register))
        .route("/auth/user/register", post(auth::user_register))
        .route("/auth/refresh", post(api::refresh_token))
        .route("/api/keys", get(api::list_api_keys).post(api::set_api_key))
        .route("/api/keys/:provider", delete(api::delete_api_key))
        .route("/api/providers/:provider/models", get(api::list_provider_models))
        .route("/api/conversations", get(api::list_conversations).post(api::create_conversation))
        .route("/api/conversations/:id", get(api::get_conversation).delete(api::delete_conversation))
        .route("/api/conversations/:id/settings", put(api::set_conversation_settings))
        .route("/api/conversations/:id/messages", get(api::get_messages).post(api::send_message))
        .route("/api/conversations/:id/stream", post(api::stream_conversation))
        .route("/api/conversations/:id/compact", post(api::compact_conversation))
        .route("/api/conversations/:id", patch(api::update_conversation))
        .route("/api/conversations/:id/system-prompt", post(api::set_conversation_system_prompt))
        .route("/api/system-prompts/templates", get(api::list_system_prompt_templates))
        .route("/api/courses", get(api::list_courses))
        .route("/api/courses/:id", get(api::get_course))
        .route("/api/courses/:id/enroll", post(api::enroll_course))
        .route("/api/enrollments", get(api::list_enrollments))
        .route("/api/enrollments/:id/progress", get(api::get_enrollment_progress))
        .route("/api/lessons/:id", get(api::get_lesson_detail))
        .route("/api/lessons/:id/start", post(api::start_lesson))
        .route("/api/lessons/:id/chat", get(api::get_lesson_chat))
        .route("/api/modules/:id/lessons", get(api::get_module_lessons))
        .route("/api/progress", post(api::update_progress))
        .route("/api/chat/stream", post(api::chat_stream))
        .route("/api/admin/users", get(auth::admin_list_users))
        .route("/api/admin/users/pending", get(auth::admin_pending_users))
        .route("/api/admin/approve-user", post(auth::admin_approve_user))
        // Backward-compatible button routes used by older static builds.
        .route("/api/admin/users/:id/approve", post(auth::admin_approve_user_path))
        .route("/api/admin/users/:id/reject", post(auth::admin_reject_user_path))
        .route("/api/admin/create-user", post(auth::admin_create_user))
        .route("/api/admin/users/:id/conversations", get(auth::admin_list_user_conversations))
        .route("/api/admin/users/:id/system-prompt", get(auth::admin_get_user_system_prompt).post(auth::admin_set_user_system_prompt))
        .route("/api/admin/conversations/:id/messages", get(auth::admin_get_conversation_messages))
        .route("/api/admin/system-prompts", get(auth::admin_list_system_prompts).post(auth::admin_create_system_prompt))
        .route("/api/admin/system-prompts/:id/activate", post(auth::admin_activate_system_prompt))
        .route("/api/admin/system-prompts/:id", delete(auth::admin_delete_system_prompt))
        .route("/api/terminal/mount", post(api::terminal_mount))
        .route("/api/terminal/sessions", get(api::terminal_list_sessions))
        .route("/api/terminal/sessions/:id", delete(api::terminal_unmount))
        .route("/api/terminal/sessions/:id/exec", post(api::terminal_exec))
        .route("/api/terminal/sessions/:id/read", post(api::terminal_read))
        .route("/api/terminal/sessions/:id/write", post(api::terminal_write))
        .route("/api/terminal/sessions/:id/ls", post(api::terminal_ls))
        .route("/api/roadmaps", get(course_api::list_roadmaps).post(course_api::create_roadmap))
        .route("/api/roadmaps/:id", get(course_api::get_roadmap).delete(course_api::delete_roadmap))
        .route("/api/roadmaps/:id/activate", post(course_api::set_active_roadmap))
        .route("/api/roadmaps/:id/topics", post(course_api::create_topic))
        .route("/api/courses/:id/create-roadmap", post(course_api::create_roadmap_from_course))
        .route("/api/topics/:id/lessons", post(course_api::create_lesson))
        .route("/api/progress/:user_id", get(course_api::get_student_progress))
        .route("/api/progress/:user_id/:lesson_id", post(course_api::update_lesson_progress))
        .route("/api/metrics", get(course_api::get_user_metrics))
        .route("/api/active-roadmap", get(course_api::get_active_roadmap))
        .route("/ws/chat", get(api::chat_websocket))
        .fallback_service(ServeDir::new(static_dir.clone()))
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::any())
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                    HeaderName::from_static("x-secret-word"),
                ]),
        )
        .with_state(state.clone());

    // Start Anthropic proxy for Claude Code integration
    let proxy_port = std::env::var("PROXY_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);
    let proxy_target = std::env::var("PROXY_TARGET")
        .unwrap_or_else(|_| "https://api.augureai.ca/v1".to_string());
    
    // Try to get the augure API key from state
    let proxy_api_key = {
        let keys = state.api_keys.read().await;
        keys.get("augure").cloned().unwrap_or_default()
    };
    
    if !proxy_api_key.is_empty() {
        let bind_localhost_only = std::env::var("PROXY_BIND_LOCALHOST_ONLY")
            .ok()
            .map(|v| !matches!(v.to_ascii_lowercase().as_str(), "false" | "0" | "no"))
            .unwrap_or(true);
        let proxy_auth_key = std::env::var("PROXY_API_KEY").ok();

        let proxy_config = anthropic_proxy::ProxyConfig {
            target_base_url: proxy_target,
            api_key: proxy_api_key,
            bind_localhost_only,
            proxy_api_key: proxy_auth_key,
        };
        
        tokio::spawn(async move {
            if let Err(e) = anthropic_proxy::run_proxy(proxy_config, proxy_port).await {
                tracing::error!("Proxy server error: {}", e);
            }
        });
        
        tracing::info!("🔄 Anthropic proxy running on http://0.0.0.0:{}", proxy_port);
        tracing::info!("   Claude Code: export ANTHROPIC_API_KEY=<augure-key> && export ANTHROPIC_BASE_URL=http://0.0.0.0:{}", proxy_port);
    } else {
        tracing::warn!("⚠️  No augure API key configured. Proxy not started.");
        tracing::info!("   Set an augure key in Settings panel to enable Claude Code proxy.");
    }

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("🚀 Almanach Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
