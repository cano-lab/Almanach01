#!/usr/bin/env python3
"""Migrate chat.db to add course/lesson tables and seed Ontario MTH1W curriculum."""
import json
import sqlite3
import sys
from pathlib import Path

DB_PATH = Path("/root/.openclaw/workspace/claw-pen/data/chat.db")
CURRICULUM_PATH = Path("/root/.openclaw/workspace/claw-pen/data/curriculum_mth1w.json")

def migrate():
    conn = sqlite3.connect(DB_PATH)
    conn.execute("PRAGMA foreign_keys = ON")
    cursor = conn.cursor()

    # Add lesson_id to conversations if not exists
    cursor.execute("SELECT COUNT(*) FROM pragma_table_info('conversations') WHERE name='lesson_id'")
    if cursor.fetchone()[0] == 0:
        cursor.execute("ALTER TABLE conversations ADD COLUMN lesson_id TEXT")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_conversations_lesson ON conversations(lesson_id)")
        print("Added conversations.lesson_id")

    # Create course tables
    tables = [
        """
        CREATE TABLE IF NOT EXISTS courses (
            id          TEXT PRIMARY KEY,
            title       TEXT NOT NULL,
            title_en    TEXT,
            description TEXT,
            grade       TEXT,
            language    TEXT DEFAULT 'fr',
            credit_hours INTEGER,
            created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        """,
        """
        CREATE TABLE IF NOT EXISTS modules (
            id          TEXT PRIMARY KEY,
            course_id   TEXT NOT NULL REFERENCES courses(id),
            title       TEXT NOT NULL,
            title_en    TEXT,
            description TEXT,
            order_index INTEGER DEFAULT 0,
            estimated_hours INTEGER,
            created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        """,
        """
        CREATE TABLE IF NOT EXISTS lessons (
            id              TEXT PRIMARY KEY,
            module_id       TEXT NOT NULL REFERENCES modules(id),
            title           TEXT NOT NULL,
            title_en        TEXT,
            description     TEXT,
            topics          TEXT,  -- JSON array
            objectives      TEXT,  -- JSON array
            estimated_minutes INTEGER DEFAULT 45,
            system_prompt   TEXT,
            keywords        TEXT,  -- JSON array
            order_index     INTEGER DEFAULT 0,
            created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        """,
        """
        CREATE TABLE IF NOT EXISTS enrollments (
            id          TEXT PRIMARY KEY,
            user_id     TEXT NOT NULL REFERENCES users(id),
            course_id   TEXT NOT NULL REFERENCES courses(id),
            status      TEXT DEFAULT 'active' CHECK(status IN ('active','completed','dropped')),
            enrolled_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            completed_at DATETIME,
            UNIQUE(user_id, course_id)
        )
        """,
        """
        CREATE TABLE IF NOT EXISTS lesson_progress (
            id              TEXT PRIMARY KEY,
            enrollment_id   TEXT NOT NULL REFERENCES enrollments(id),
            lesson_id       TEXT NOT NULL REFERENCES lessons(id),
            status          TEXT DEFAULT 'not_started' CHECK(status IN ('not_started','in_progress','completed')),
            started_at      DATETIME,
            completed_at    DATETIME,
            last_activity_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(enrollment_id, lesson_id)
        )
        """,
    ]

    for sql in tables:
        cursor.execute(sql)

    # Seed curriculum data
    if not CURRICULUM_PATH.exists():
        print(f"Curriculum file not found: {CURRICULUM_PATH}")
        sys.exit(1)

    with open(CURRICULUM_PATH) as f:
        data = json.load(f)

    course = data["course"]

    # Insert course
    cursor.execute(
        "INSERT OR IGNORE INTO courses (id, title, title_en, description, grade, language, credit_hours) VALUES (?, ?, ?, ?, ?, ?, ?)",
        (course["id"], course["title"], course.get("title_en"), course["description"], course.get("grade"), course.get("language", "fr"), course.get("credit_hours")),
    )

    for module in course["modules"]:
        cursor.execute(
            "INSERT OR IGNORE INTO modules (id, course_id, title, title_en, description, order_index, estimated_hours) VALUES (?, ?, ?, ?, ?, ?, ?)",
            (module["id"], course["id"], module["title"], module.get("title_en"), module.get("description"), module["order"], module.get("estimated_hours")),
        )

        for lesson in module["lessons"]:
            cursor.execute(
                """INSERT OR IGNORE INTO lessons
                (id, module_id, title, title_en, description, topics, objectives, estimated_minutes, system_prompt, keywords, order_index)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    lesson["id"],
                    module["id"],
                    lesson["title"],
                    lesson.get("title_en"),
                    lesson.get("description"),
                    json.dumps(lesson.get("topics", []), ensure_ascii=False),
                    json.dumps(lesson.get("objectives", []), ensure_ascii=False),
                    lesson.get("estimated_minutes", 45),
                    lesson.get("system_prompt"),
                    json.dumps(lesson.get("keywords", []), ensure_ascii=False),
                    lesson["order"],
                ),
            )

    conn.commit()
    conn.close()
    print("Migration and seeding complete!")
    print(f"  Course: {course['title']}")
    print(f"  Modules: {len(course['modules'])}")
    print(f"  Lessons: {sum(len(m['lessons']) for m in course['modules'])}")

if __name__ == "__main__":
    migrate()
