//! Course seed migration for Almanach
//!
//! Run with: cargo run --bin seed-courses

use anyhow::{Context, Result};
use rusqlite::Connection;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let db_path = args.get(1).map(|s| s.as_str()).unwrap_or("data/chat.db");

    println!("🌱 Seeding Almanach courses into {}...", db_path);

    let conn = Connection::open(db_path)
        .with_context(|| format!("opening database at {}", db_path))?;

    // Ensure foreign keys are on
    conn.pragma_update(None, "foreign_keys", "ON")?;

    // Check if courses already exist
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM courses WHERE code = 'MTH1W'",
        [],
        |row| row.get(0),
    )?;

    if count > 0 {
        println!("⚠️  MTH1W already seeded ({} courses found). Skipping.", count);
        return Ok(());
    }

    // Read and execute seed SQL
    let seed_sql = include_str!("../../migrations/seed_mth1w.sql");
    conn.execute_batch(seed_sql)
        .context("executing seed SQL")?;

    // Verify
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

    println!("✅ Seeded successfully:");
    println!("   Courses:  {}", course_count);
    println!("   Modules:  {}", module_count);
    println!("   Lessons:  {}", lesson_count);

    Ok(())
}
