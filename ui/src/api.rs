use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use crate::types::*;

// === Auth Types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[allow(dead_code)]
    pub refresh_token: String,
    #[allow(dead_code)]
    pub token_type: String,
    #[allow(dead_code)]
    pub expires_in: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthStatus {
    pub auth_enabled: bool,
    pub has_admin: bool,
    pub registration_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MeResponse {
    pub user_id: Option<String>,
    pub username: String,
    pub display_name: Option<String>,
    pub role: String,
}

// === Token Storage ────────────────────────────────────────────────────────

pub fn store_token(token: &str) {
    let window = web_sys::window().unwrap();
    let storage = window.local_storage().unwrap().unwrap();
    storage.set_item("almanach_token", token).unwrap();
}

pub fn get_token() -> Option<String> {
    let window = web_sys::window().unwrap();
    let storage = window.local_storage().unwrap().unwrap();
    storage.get_item("almanach_token").unwrap()
}

pub fn clear_token() {
    let window = web_sys::window().unwrap();
    let storage = window.local_storage().unwrap().unwrap();
    storage.remove_item("almanach_token").unwrap();
}

fn auth_header() -> Option<(&'static str, String)> {
    get_token().map(|t| ("Authorization", format!("Bearer {}", t)))
}

// === Auth API ─────────────────────────────────────────────────────────────

pub async fn get_auth_status() -> Result<AuthStatus, String> {
    let response = Request::get("/auth/status")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("API error: {}", response.status()))
    }
}

pub async fn login(password: &str) -> Result<TokenResponse, String> {
    let response = Request::post("/auth/login")
        .json(&LoginRequest { password: password.to_string() })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Login failed: {}", error_text))
    }
}

pub async fn register(password: &str) -> Result<(), String> {
    let response = Request::post("/auth/register")
        .json(&LoginRequest { password: password.to_string() })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.ok() {
        Ok(())
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Registration failed: {}", error_text))
    }
}

pub async fn me() -> Result<MeResponse, String> {
    let mut req = Request::get("/api/me");
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;
    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Me failed: {}", response.status()))
    }
}

// === Legacy Stubs (to be removed when components are replaced) ────────────

pub async fn fetch_agents() -> Result<Vec<AgentContainer>, String> {
    Ok(vec![])
}

pub async fn fetch_teams() -> Result<Vec<Team>, String> {
    Ok(vec![])
}

pub async fn fetch_team_roles(_team_id: &str) -> Result<std::collections::HashMap<String, TeamRoleAssignment>, String> {
    Ok(std::collections::HashMap::new())
}

pub async fn assign_team_role(_team_id: &str, _agent_id: &str, _role: &str) -> Result<(), String> {
    Ok(())
}

pub async fn remove_team_role(_team_id: &str, _agent_id: &str) -> Result<(), String> {
    Ok(())
}

pub async fn start_agent(_id: &str) -> Result<(), String> {
    Ok(())
}

pub async fn stop_agent(_id: &str) -> Result<(), String> {
    Ok(())
}

// === Admin API ────────────────────────────────────────────────────────────

pub async fn fetch_users() -> Result<Vec<serde_json::Value>, String> {
    let mut req = Request::get("/api/admin/users");
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;
    if response.ok() {
        let val: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        Ok(val.get("users").and_then(|u| u.as_array()).cloned().unwrap_or_default())
    } else {
        Err(format!("Failed to fetch users: {}", response.status()))
    }
}

pub async fn fetch_pending_users() -> Result<Vec<serde_json::Value>, String> {
    let mut req = Request::get("/api/admin/users/pending");
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;
    if response.ok() {
        let val: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        Ok(val.get("users").and_then(|u| u.as_array()).cloned().unwrap_or_default())
    } else {
        Err(format!("Failed to fetch pending users: {}", response.status()))
    }
}

pub async fn approve_user(user_id: &str, action: &str) -> Result<serde_json::Value, String> {
    let mut req = Request::post("/api/admin/approve-user");
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let body = serde_json::json!({
        "user_id": user_id,
        "action": action,
    });
    let response = req.json(&body).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to approve user: {}", response.status()))
    }
}

pub async fn create_user(username: &str, password: &str, display_name: Option<&str>, role: &str) -> Result<serde_json::Value, String> {
    let mut req = Request::post("/api/admin/create-user");
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let body = serde_json::json!({
        "username": username,
        "password": password,
        "display_name": display_name,
        "role": role,
    });
    let response = req.json(&body).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to create user: {}", response.status()))
    }
}

// === API Keys ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ApiKeyInfo {
    pub provider: String,
    pub has_key: bool,
}

pub async fn fetch_api_keys() -> Result<Vec<ApiKeyInfo>, String> {
    let mut req = Request::get("/api/keys");
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;
    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch API keys: {}", response.status()))
    }
}

pub async fn set_api_key(provider: &str, key: &str) -> Result<(), String> {
    let mut req = Request::post("/api/keys");
    if let Some((key_hdr, val)) = auth_header() {
        req = req.header(key_hdr, &val);
    }
    let body = serde_json::json!({ "provider": provider, "key": key });
    let response = req.json(&body).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if response.ok() {
        Ok(())
    } else {
        Err(format!("Failed to set API key: {}", response.status()))
    }
}

pub async fn delete_api_key(provider: &str) -> Result<(), String> {
    let mut req = Request::delete(&format!("/api/keys/{}", provider));
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;
    if response.ok() {
        Ok(())
    } else {
        Err(format!("Failed to delete API key: {}", response.status()))
    }
}

// === Course API ───────────────────────────────────────────────────────────

pub async fn fetch_courses() -> Result<Vec<Course>, String> {
    let response = Request::get("/api/courses")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch courses: {}", response.status()))
    }
}

pub async fn get_course(id: &str) -> Result<serde_json::Value, String> {
    let response = Request::get(&format!("/api/courses/{}", id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch course: {}", response.status()))
    }
}

pub async fn enroll_course(id: &str) -> Result<serde_json::Value, String> {
    let mut req = Request::post(&format!("/api/courses/{}/enroll", id));
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to enroll: {}", response.status()))
    }
}

// === Enrollment API ───────────────────────────────────────────────────────

pub async fn fetch_enrollments() -> Result<Vec<Enrollment>, String> {
    let mut req = Request::get("/api/enrollments");
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch enrollments: {}", response.status()))
    }
}

pub async fn get_enrollment_progress(enrollment_id: &str) -> Result<Vec<LessonProgress>, String> {
    let mut req = Request::get(&format!("/api/enrollments/{}/progress", enrollment_id));
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        let val: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let progress = val.get("progress")
            .and_then(|p| p.as_array())
            .map(|arr| arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect())
            .unwrap_or_default();
        Ok(progress)
    } else {
        Err(format!("Failed to fetch progress: {}", response.status()))
    }
}

// === Lesson API ───────────────────────────────────────────────────────────

pub async fn get_lesson(id: &str) -> Result<LessonDetail, String> {
    let response = Request::get(&format!("/api/lessons/{}", id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch lesson: {}", response.status()))
    }
}

pub async fn start_lesson_chat(id: &str) -> Result<serde_json::Value, String> {
    let mut req = Request::post(&format!("/api/lessons/{}/start", id));
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to start lesson: {}", response.status()))
    }
}

pub async fn get_lesson_chat(id: &str) -> Result<Option<Conversation>, String> {
    let mut req = Request::get(&format!("/api/lessons/{}/chat", id));
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        let val: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        if val.get("has_chat").and_then(|v| v.as_bool()).unwrap_or(false) {
            let conv = val.get("conversation_id")
                .and_then(|id| id.as_str())
                .map(|id| Conversation {
                    id: id.to_string(),
                    title: val.get("title").and_then(|t| t.as_str()).unwrap_or("Chat").to_string(),
                    system_prompt: None,
                    created_at: String::new(),
                });
            Ok(conv)
        } else {
            Ok(None)
        }
    } else {
        Err(format!("Failed to fetch lesson chat: {}", response.status()))
    }
}

pub async fn update_progress(enrollment_id: &str, lesson_id: &str, status: &str) -> Result<serde_json::Value, String> {
    let mut req = Request::post("/api/progress");
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let body = serde_json::json!({
        "enrollment_id": enrollment_id,
        "lesson_id": lesson_id,
        "status": status,
    });
    let response = req.json(&body).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to update progress: {}", response.status()))
    }
}

// === Chat API ─────────────────────────────────────────────────────────────

pub async fn fetch_messages(conversation_id: &str) -> Result<Vec<ChatMessage>, String> {
    let mut req = Request::get(&format!("/api/conversations/{}/messages", conversation_id));
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch messages: {}", response.status()))
    }
}

pub async fn send_message(conversation_id: &str, content: &str) -> Result<ChatMessage, String> {
    let mut req = Request::post(&format!("/api/conversations/{}/messages", conversation_id));
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let body = serde_json::json!({ "role": "user", "content": content });
    let response = req.json(&body).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to send message: {}", response.status()))
    }
}

// === Terminal Agent API ───────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TerminalSession {
    pub id: String,
    pub path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileContent {
    pub content: String,
    pub size: usize,
    pub is_directory: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub entry_type: String,
    pub size: usize,
}

pub async fn mount_directory(path: &str) -> Result<TerminalSession, String> {
    let mut req = Request::post("/api/terminal/mount");
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let body = serde_json::json!({ "path": path });
    let response = req.json(&body).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to mount directory: {}", response.status()))
    }
}

pub async fn list_terminal_sessions() -> Result<Vec<TerminalSession>, String> {
    let mut req = Request::get("/api/terminal/sessions");
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        let val: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let sessions = val.get("sessions")
            .and_then(|s| s.as_array())
            .map(|arr| arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect())
            .unwrap_or_default();
        Ok(sessions)
    } else {
        Err(format!("Failed to list sessions: {}", response.status()))
    }
}

pub async fn unmount_directory(session_id: &str) -> Result<(), String> {
    let mut req = Request::delete(&format!("/api/terminal/sessions/{}", session_id));
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;

    if response.ok() || response.status() == 204 {
        Ok(())
    } else {
        Err(format!("Failed to unmount: {}", response.status()))
    }
}

pub async fn exec_command(session_id: &str, command: &str) -> Result<ExecResult, String> {
    let mut req = Request::post(&format!("/api/terminal/sessions/{}/exec", session_id));
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let body = serde_json::json!({ "command": command });
    let response = req.json(&body).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to execute command: {}", response.status()))
    }
}

pub async fn read_file(session_id: &str, path: &str) -> Result<FileContent, String> {
    let mut req = Request::post(&format!("/api/terminal/sessions/{}/read", session_id));
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let body = serde_json::json!({ "path": path });
    let response = req.json(&body).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to read file: {}", response.status()))
    }
}

pub async fn write_file(session_id: &str, path: &str, content: &str) -> Result<serde_json::Value, String> {
    let mut req = Request::post(&format!("/api/terminal/sessions/{}/write", session_id));
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let body = serde_json::json!({ "path": path, "content": content });
    let response = req.json(&body).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to write file: {}", response.status()))
    }
}

pub async fn list_dir(session_id: &str, path: &str) -> Result<Vec<DirEntry>, String> {
    let mut req = Request::post(&format!("/api/terminal/sessions/{}/ls", session_id));
    if let Some((key, val)) = auth_header() {
        req = req.header(key, &val);
    }
    let body = serde_json::json!({ "path": path });
    let response = req.json(&body).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        let val: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let entries = val.get("entries")
            .and_then(|e| e.as_array())
            .map(|arr| arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect())
            .unwrap_or_default();
        Ok(entries)
    } else {
        Err(format!("Failed to list directory: {}", response.status()))
    }
}
