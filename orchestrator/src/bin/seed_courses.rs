//! Course seed migration for Almanach
//!
//! Run with: cargo run --bin seed-courses

use anyhow::{Context, Result};
use rusqlite::Connection;

const SCHEMA_COURSES: &str = r#"
CREATE TABLE IF NOT EXISTS courses (
    id              TEXT PRIMARY KEY,
    code            TEXT UNIQUE NOT NULL,
    title           TEXT NOT NULL,
    title_en        TEXT,
    description     TEXT,
    grade           TEXT,
    language        TEXT NOT NULL DEFAULT 'fr',
    credit_hours    INTEGER,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_courses_code ON courses(code);

CREATE TABLE IF NOT EXISTS modules (
    id              TEXT PRIMARY KEY,
    course_id       TEXT NOT NULL,
    title           TEXT NOT NULL,
    title_en        TEXT,
    description     TEXT,
    order_index     INTEGER NOT NULL DEFAULT 0,
    estimated_hours INTEGER,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(course_id) REFERENCES courses(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_modules_course ON modules(course_id, order_index);

CREATE TABLE IF NOT EXISTS lessons (
    id                  TEXT PRIMARY KEY,
    module_id           TEXT NOT NULL,
    title               TEXT NOT NULL,
    title_en            TEXT,
    description         TEXT,
    topics              TEXT DEFAULT '[]',
    objectives          TEXT DEFAULT '[]',
    estimated_minutes   INTEGER NOT NULL DEFAULT 60,
    system_prompt       TEXT,
    keywords            TEXT DEFAULT '[]',
    order_index         INTEGER NOT NULL DEFAULT 0,
    created_at          DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(module_id) REFERENCES modules(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_lessons_module ON lessons(module_id, order_index);
"#;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let db_path = args.get(1).map(|s| s.as_str()).unwrap_or("data/chat.db");

    println!("🌱 Seeding Almanach courses into {}...", db_path);

    let conn = Connection::open(db_path)
        .with_context(|| format!("opening database at {}", db_path))?;

    // Ensure foreign keys are on
    conn.pragma_update(None, "foreign_keys", "ON")?;

    // If an older/stale courses table exists without the `code` column, drop and recreate
    let courses_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='courses'",
        [],
        |row| row.get(0),
    )?;
    if courses_exists > 0 {
        let has_code: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('courses') WHERE name = 'code'",
            [],
            |row| row.get(0),
        )?;
        if has_code == 0 {
            println!("⚠️  Existing courses table is missing the 'code' column. Recreating course tables...");
            conn.execute_batch(
                "DROP TABLE IF EXISTS lessons;
                 DROP TABLE IF EXISTS modules;
                 DROP TABLE IF EXISTS courses;"
            )?;
        }
    }

    // Ensure course schema exists before seeding
    conn.execute_batch(SCHEMA_COURSES)?;

    let seeds = [
        ("MTH1W", include_str!("../../migrations/seed_mth1w.sql")),
        ("ICD20", include_str!("../../migrations/seed_ics2o.sql")),
    ];

    for (code, sql) in &seeds {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM courses WHERE code = ?1",
            [code],
            |row| row.get(0),
        )?;

        if count > 0 {
            println!("⚠️  {} already seeded ({} courses found). Skipping.", code, count);
            continue;
        }

        conn.execute_batch(sql)
            .with_context(|| format!("executing seed SQL for {}", code))?;

        println!("✅ Seeded {} successfully.", code);
    }

    // Verify totals
    let course_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM courses",
        [],
        |row| row.get(0),
    )?;
    let module_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM modules",
        [],
        |row| row.get(0),
    )?;
    let lesson_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM lessons",
        [],
        |row| row.get(0),
    )?;

    println!("📊 Totals in database:");
    println!("   Courses:  {}", course_count);
    println!("   Modules:  {}", module_count);
    println!("   Lessons:  {}", lesson_count);

    Ok(())
}
