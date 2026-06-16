use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use crate::types::*;

// === Auth & Token Storage ─────────────────────────────────────────────────

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
pub struct MeResponse {
    pub username: String,
    pub role: String,
}

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
