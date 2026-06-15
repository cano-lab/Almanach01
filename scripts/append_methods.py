#!/usr/bin/env python3
"""Append course methods to chat_db.rs"""
import sys

FILE = "/root/.openclaw/workspace/claw-pen/orchestrator/src/chat_db.rs"

# Find the line with "// ─── Roadmap (legacy)"
with open(FILE) as f:
    lines = f.readlines()

insert_idx = None
for i, line in enumerate(lines):
    if "// ─── Roadmap (legacy)" in line:
        insert_idx = i
        break

if insert_idx is None:
    print("Could not find insertion point")
    sys.exit(1)

new_methods = '''
    // ─── Courses ────────────────────────────────────────────────────────────

    pub fn list_courses(&self) -> Result<Vec<Course>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, title_en, description, grade, language, credit_hours, created_at
             FROM courses
             ORDER BY title"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Course {
                id: row.get(0)?,
                title: row.get(1)?,
                title_en: row.get(2).ok(),
                description: row.get(3)?,
                grade: row.get(4)?,
                language: row.get(5)?,
                credit_hours: row.get(6).ok(),
                created_at: row.get(7)?,
            })
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn get_course(&self, id: &str) -> Result<Option<Course>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, title, title_en, description, grade, language, credit_hours, created_at
             FROM courses WHERE id = ?1",
            params![id],
            |row| {
                Ok(Course {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    title_en: row.get(2).ok(),
                    description: row.get(3)?,
                    grade: row.get(4)?,
                    language: row.get(5)?,
                    credit_hours: row.get(6).ok(),
                    created_at: row.get(7)?,
                })
            },
        ).optional().context("get_course")
    }

    pub fn get_course_modules(&self, course_id: &str) -> Result<Vec<Module>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, course_id, title, title_en, description, order_index, estimated_hours, created_at
             FROM modules WHERE course_id = ?1 ORDER BY order_index"
        )?;
        let rows = stmt.query_map(params![course_id], |row| {
            Ok(Module {
                id: row.get(0)?,
                course_id: row.get(1)?,
                title: row.get(2)?,
                title_en: row.get(3).ok(),
                description: row.get(4)?,
                order_index: row.get(5)?,
                estimated_hours: row.get(6).ok(),
                created_at: row.get(7)?,
                lessons: Vec::new(),
            })
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn get_module_lessons(&self, module_id: &str) -> Result<Vec<Lesson>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, module_id, title, title_en, description, topics, objectives, estimated_minutes, system_prompt, keywords, order_index, created_at
             FROM lessons WHERE module_id = ?1 ORDER BY order_index"
        )?;
        let rows = stmt.query_map(params![module_id], |row| {
            let topics_json: String = row.get(5)?;
            let objectives_json: String = row.get(6)?;
            let keywords_json: String = row.get(9)?;
            Ok(Lesson {
                id: row.get(0)?,
                module_id: row.get(1)?,
                title: row.get(2)?,
                title_en: row.get(3).ok(),
                description: row.get(4)?,
                topics: serde_json::from_str(&topics_json).unwrap_or_default(),
                objectives: serde_json::from_str(&objectives_json).unwrap_or_default(),
                estimated_minutes: row.get(7)?,
                system_prompt: row.get(8).unwrap_or_default(),
                keywords: serde_json::from_str(&keywords_json).unwrap_or_default(),
                order_index: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn enroll_user(&self, user_id: &str, course_id: &str) -> Result<Enrollment> {
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO enrollments (id, user_id, course_id, status, enrolled_at)
             VALUES (?1, ?2, ?3, 'active', CURRENT_TIMESTAMP)
             ON CONFLICT(user_id, course_id) DO UPDATE SET status = 'active'",
            params![id, user_id, course_id],
        )?;
        Ok(Enrollment {
            id,
            user_id: user_id.to_string(),
            course_id: course_id.to_string(),
            status: "active".to_string(),
            enrolled_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
        })
    }

    pub fn list_enrollments(&self, user_id: &str) -> Result<Vec<Enrollment>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, course_id, status, enrolled_at, completed_at
             FROM enrollments WHERE user_id = ?1 ORDER BY enrolled_at DESC"
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok(Enrollment {
                id: row.get(0)?,
                user_id: row.get(1)?,
                course_id: row.get(2)?,
                status: row.get(3)?,
                enrolled_at: row.get(4)?,
                completed_at: row.get(5).ok(),
            })
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn get_lesson_progress(&self, enrollment_id: &str, lesson_id: &str) -> Result<Option<LessonProgress>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, enrollment_id, lesson_id, status, started_at, completed_at, last_activity_at
             FROM lesson_progress WHERE enrollment_id = ?1 AND lesson_id = ?2",
            params![enrollment_id, lesson_id],
            |row| {
                Ok(LessonProgress {
                    id: row.get(0)?,
                    enrollment_id: row.get(1)?,
                    lesson_id: row.get(2)?,
                    status: row.get(3)?,
                    started_at: row.get(4).ok(),
                    completed_at: row.get(5).ok(),
                    last_activity_at: row.get(6)?,
                })
            },
        ).optional().context("get_lesson_progress")
    }

    pub fn update_lesson_progress(&self, enrollment_id: &str, lesson_id: &str, status: &str) -> Result<LessonProgress> {
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let started_at = if status == "in_progress" || status == "completed" { Some(&now) } else { None };
        let completed_at = if status == "completed" { Some(&now) } else { None };

        conn.execute(
            "INSERT INTO lesson_progress (id, enrollment_id, lesson_id, status, started_at, completed_at, last_activity_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(enrollment_id, lesson_id) DO UPDATE SET
                 status = excluded.status,
                 started_at = COALESCE(excluded.started_at, lesson_progress.started_at),
                 completed_at = excluded.completed_at,
                 last_activity_at = excluded.last_activity_at",
            params![id, enrollment_id, lesson_id, status, started_at, completed_at, now],
        )?;

        Ok(LessonProgress {
            id,
            enrollment_id: enrollment_id.to_string(),
            lesson_id: lesson_id.to_string(),
            status: status.to_string(),
            started_at: started_at.map(|s| s.to_string()),
            completed_at: completed_at.map(|s| s.to_string()),
            last_activity_at: now,
        })
    }

    pub fn create_lesson_conversation(&self, user_id: &str, lesson_id: &str, title: &str, system_prompt: &str) -> Result<ChatConversation> {
        let colors = ["#2a7f7f", "#b85c38", "#d4a843", "#5a7a5a", "#8a6a8a"];
        let color = colors[uuid::Uuid::new_v4().as_u128() as usize % colors.len()];
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO conversations (id, user_id, agent_id, title, color, system_prompt, lesson_id, created_at, last_message_at)
             VALUES (?1, ?2, 'chat', ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![id, user_id, title, color, system_prompt, lesson_id],
        )?;
        Ok(ChatConversation {
            id,
            title: title.to_string(),
            system_prompt: Some(system_prompt.to_string()),
            color: color.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            message_count: 0,
        })
    }

    pub fn get_conversation_by_lesson(&self, user_id: &str, lesson_id: &str) -> Result<Option<ChatConversation>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, COALESCE(title, 'New Chat'), system_prompt, COALESCE(color, '#2a7f7f'), created_at, last_message_at,
                    (SELECT COUNT(*) FROM messages WHERE conversation_id = c.id) as msg_count
             FROM conversations c
             WHERE user_id = ?1 AND lesson_id = ?2
             ORDER BY last_message_at DESC LIMIT 1",
            params![user_id, lesson_id],
            |row| {
                Ok(ChatConversation {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    system_prompt: row.get(2).ok(),
                    color: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5).unwrap_or_default(),
                    message_count: row.get(6)?,
                })
            },
        ).optional().context("get_conversation_by_lesson")
    }

    pub fn get_lesson(&self, id: &str) -> Result<Option<Lesson>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, module_id, title, title_en, description, topics, objectives, estimated_minutes, system_prompt, keywords, order_index, created_at
             FROM lessons WHERE id = ?1",
            params![id],
            |row| {
                let topics_json: String = row.get(5)?;
                let objectives_json: String = row.get(6)?;
                let keywords_json: String = row.get(9)?;
                Ok(Lesson {
                    id: row.get(0)?,
                    module_id: row.get(1)?,
                    title: row.get(2)?,
                    title_en: row.get(3).ok(),
                    description: row.get(4)?,
                    topics: serde_json::from_str(&topics_json).unwrap_or_default(),
                    objectives: serde_json::from_str(&objectives_json).unwrap_or_default(),
                    estimated_minutes: row.get(7)?,
                    system_prompt: row.get(8).unwrap_or_default(),
                    keywords: serde_json::from_str(&keywords_json).unwrap_or_default(),
                    order_index: row.get(10)?,
                    created_at: row.get(11)?,
                })
            },
        ).optional().context("get_lesson")
    }
'''

# Insert before the roadmap line
new_lines = lines[:insert_idx] + [new_methods] + lines[insert_idx:]

with open(FILE, 'w') as f:
    f.writelines(new_lines)

print(f"Inserted {len(new_methods.splitlines())} lines at line {insert_idx}")
