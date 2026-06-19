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

use axum::http::{header, HeaderName, HeaderValue, Method, StatusCode};
use axum::{
    routing::{get, post, delete, patch, put},
    Router,
    extract::Path,
    middleware,
};
use secrecy::{ExposeSecret, SecretString};
use tokio::sync::RwLock;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::GlobalKeyExtractor;
use tower_governor::GovernorLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

use crate::auth::AuthManager;
use crate::chat_db::ChatDb;

pub struct AppState {
    pub config: config::Config,
    pub api_keys: HashMap<String, SecretString>,
    pub data_dir: std::path::PathBuf,
    pub auth: RwLock<AuthManager>,
    pub chat_db: Arc<ChatDb>,
    pub terminal: terminal_agent::TerminalAgent,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--set-password".to_string()) {
        let data_dir = std::path::PathBuf::from("./data");
        auth::cli_set_password(&data_dir)?;
        return Ok(());
    }

    if args.contains(&"--migrate-api-keys".to_string()) {
        let data_dir = std::path::PathBuf::from("./data");
        let keys_path = data_dir.join("api_keys.json");
        if keys_path.exists() {
            let contents = std::fs::read_to_string(&keys_path)?;
            let keys: HashMap<String, String> = serde_json::from_str(&contents)?;
            println!("# Add these lines to your .env file or environment:");
            for (provider, key) in keys {
                println!("{}_API_KEY={}", provider.to_uppercase(), key);
            }
            println!("\n# Then delete {}", keys_path.display());
        } else {
            println!("No api_keys.json found at {}", keys_path.display());
        }
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

    let api_keys = load_api_keys_from_env();
    let chat_db = Arc::new(ChatDb::open(&data_dir.join("chat.db"))?);

    let state = Arc::new(AppState {
        config: config.clone(),
        api_keys,
        data_dir: data_dir.clone(),
        auth: RwLock::new(auth_manager),
        chat_db,
        terminal: terminal_agent::TerminalAgent::new(),
    });

    // CORS configuration
    let allow_any = std::env::var("CORS_ALLOW_ANY")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let allow_origin = if allow_any {
        tracing::warn!("CORS_ALLOW_ANY=true: allowing requests from any origin (not recommended in production)");
        AllowOrigin::any()
    } else {
        let origins: Vec<HeaderValue> = config
            .cors_allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        AllowOrigin::list(origins)
    };

    let cors_layer = CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            HeaderName::from_static("x-secret-word"),
            HeaderName::from_static("x-requested-with"),
        ]);

    // Sensitive endpoints get stricter rate limits
    let sensitive_limit = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(5)
            .key_extractor(GlobalKeyExtractor)
            .use_headers()
            .finish()
            .unwrap(),
    );
    let sensitive_router = Router::new()
        .route("/auth/login", post(api::login))
        .route("/auth/user/login", post(auth::user_login))
        .route("/auth/register", post(api::register))
        .route("/auth/user/register", post(auth::user_register))
        .route("/api/chat/stream", post(api::chat_stream))
        .route("/api/conversations/:id/stream", post(api::stream_conversation))
        .layer(GovernorLayer {
            config: sensitive_limit,
        });

    // Global rate limit
    let global_limit = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(10)
            .burst_size(50)
            .key_extractor(GlobalKeyExtractor)
            .use_headers()
            .finish()
            .unwrap(),
    );

    let app = Router::new()
        .merge(sensitive_router)
        .route("/health", get(api::health))
        .route("/api/me", get(auth::me))
        .route("/auth/me", get(auth::me))
        .route("/auth/refresh", post(api::refresh_token))
        .route("/api/keys", get(api::list_api_keys).post(api::set_api_key))
        .route("/api/keys/:provider", delete(api::delete_api_key))
        .route("/api/providers/:provider/models", get(api::list_provider_models))
        .route("/api/conversations", get(api::list_conversations).post(api::create_conversation))
        .route("/api/conversations/:id", get(api::get_conversation).delete(api::delete_conversation))
        .route("/api/conversations/:id/settings", put(api::set_conversation_settings))
        .route("/api/conversations/:id/messages", get(api::get_messages).post(api::send_message))
        .route("/api/conversations/:id/compact", post(api::compact_conversation))
        .route("/api/conversations/:id", patch(api::update_conversation))
        .route("/api/conversations/:id/system-prompt", post(api::set_conversation_system_prompt))
        .route("/api/system-prompts/templates", get(api::list_system_prompt_templates))
        .route("/api/courses", get(api::list_courses))
        .route("/api/courses/:id", get(api::get_course))
        .route("/api/courses/:id/create-roadmap", post(course_api::create_roadmap_from_course))
        .route("/api/courses/:id/enroll", post(api::enroll_course))
        .route("/api/enrollments", get(api::list_enrollments))
        .route("/api/enrollments/:id/progress", get(api::get_enrollment_progress))
        .route("/api/lessons/:id", get(api::get_lesson_detail))
        .route("/api/lessons/:id/start", post(api::start_lesson))
        .route("/api/lessons/:id/chat", get(api::get_lesson_chat))
        .route("/api/modules/:id/lessons", get(api::get_module_lessons))
        .route("/api/progress", post(api::update_progress))
        .route("/api/admin/users", get(auth::admin_list_users))
        .route("/api/admin/users/pending", get(auth::admin_pending_users))
        .route("/api/admin/approve-user", post(auth::admin_approve_user))
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
        .route("/api/topics/:id/lessons", post(course_api::create_lesson))
        .route("/api/progress/:user_id", get(course_api::get_student_progress))
        .route("/api/progress/:user_id/:lesson_id", post(course_api::update_lesson_progress))
        .route("/api/metrics", get(course_api::get_user_metrics))
        .route("/api/active-roadmap", get(course_api::get_active_roadmap))
        .route("/ws/chat", get(api::chat_websocket))
        .fallback_service(ServeDir::new("./static-site"))
        .layer(middleware::from_fn_with_state(state.clone(), auth::auth_middleware))
        .layer(cors_layer)
        .layer(GovernorLayer { config: global_limit })
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    // Start Anthropic proxy for Claude Code integration
    let proxy_port = std::env::var("PROXY_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);
    let proxy_target = std::env::var("PROXY_TARGET")
        .unwrap_or_else(|_| "https://api.augureai.ca/v1".to_string());

    let proxy_bind_localhost_only = std::env::var("PROXY_BIND_LOCALHOST_ONLY")
        .map(|v| !v.eq_ignore_ascii_case("false"))
        .unwrap_or(true);
    let proxy_api_key = std::env::var("PROXY_API_KEY").ok();

    if !proxy_bind_localhost_only && proxy_api_key.is_none() {
        anyhow::bail!("PROXY_API_KEY must be set when PROXY_BIND_LOCALHOST_ONLY=false");
    }

    let augure_key = state
        .api_keys
        .get("augure")
        .map(|s| s.expose_secret().clone())
        .unwrap_or_default();

    if !augure_key.is_empty() {
        let proxy_config = anthropic_proxy::ProxyConfig {
            target_base_url: proxy_target,
            api_key: augure_key,
            bind_localhost_only: proxy_bind_localhost_only,
            proxy_api_key,
        };
        let bind_addr = if proxy_bind_localhost_only {
            format!("127.0.0.1:{}", proxy_port)
        } else {
            format!("0.0.0.0:{}", proxy_port)
        };

        tokio::spawn(async move {
            if let Err(e) = anthropic_proxy::run_proxy(proxy_config, proxy_port).await {
                tracing::error!("Proxy server error: {}", e);
            }
        });

        tracing::info!("🔄 Anthropic proxy running on http://{}", bind_addr);
        if proxy_bind_localhost_only {
            tracing::info!("   Claude Code: export ANTHROPIC_API_KEY=<augure-key> && export ANTHROPIC_BASE_URL=http://127.0.0.1:{}", proxy_port);
        } else {
            tracing::info!("   Proxy is exposed to all interfaces; PROXY_API_KEY authentication is enabled.");
        }
    } else {
        tracing::warn!("⚠️  No augure API key configured. Proxy not started.");
        tracing::info!("   Set AUGURE_API_KEY in your environment to enable the Claude Code proxy.");
    }

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("🚀 Almanach Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn load_api_keys_from_env() -> HashMap<String, SecretString> {
    let mut keys = HashMap::new();
    let providers = ["anthropic", "openai", "kimi", "google", "augure", "zai"];
    for provider in providers {
        let env_var = format!("{}_API_KEY", provider.to_uppercase());
        if let Ok(key) = std::env::var(&env_var) {
            if !key.is_empty() {
                keys.insert(provider.to_string(), SecretString::new(key.into()));
            }
        }
    }
    keys
}
