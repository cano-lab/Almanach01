use serde::{Deserialize, Serialize};

// ─── Course Types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Course {
    pub id: String,
    pub title: String,
    pub title_en: Option<String>,
    pub description: String,
    pub grade: String,
    pub language: String,
    pub credit_hours: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Module {
    pub id: String,
    pub course_id: String,
    pub title: String,
    pub title_en: Option<String>,
    pub description: String,
    pub order_index: i64,
    pub estimated_hours: Option<i64>,
    #[serde(default)]
    pub lessons: Vec<Lesson>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub title_en: Option<String>,
    pub description: String,
    pub estimated_minutes: i64,
    pub order_index: i64,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub objectives: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LessonDetail {
    pub id: String,
    pub title: String,
    pub title_en: Option<String>,
    pub description: String,
    pub topics: Vec<String>,
    pub objectives: Vec<String>,
    pub estimated_minutes: i64,
    pub keywords: Vec<String>,
    pub order_index: i64,
}

// ─── Enrollment & Progress ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Enrollment {
    pub id: String,
    pub course_id: String,
    pub status: String,
    pub enrolled_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LessonProgress {
    pub id: String,
    pub enrollment_id: String,
    pub lesson_id: String,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub last_activity_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnrollmentWithProgress {
    pub enrollment: Enrollment,
    pub course: Option<Course>,
    pub progress: Vec<LessonProgress>,
    pub total_lessons: usize,
    pub completed_lessons: usize,
}

// ─── Chat Types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub system_prompt: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

// ─── User Types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub role: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MeResponse {
    pub username: String,
    pub role: String,
}
