#!/usr/bin/env bash
# Seed Almanach course catalog into an SQLite database.
# Run from the orchestrator directory, or pass the database path as an argument.
#
# Usage:
#   ./migrations/seed_courses.sh
#   ./migrations/seed_courses.sh /path/to/chat.db

set -euo pipefail

DB_PATH="${1:-data/chat.db}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if ! command -v sqlite3 &> /dev/null; then
    echo "❌ sqlite3 is required but not installed."
    echo "   Install it with: sudo apt install sqlite3"
    exit 1
fi

echo "🌱 Seeding Almanach courses into ${DB_PATH}..."

# Ensure the database directory exists
mkdir -p "$(dirname "${DB_PATH}")"

# If an older courses table exists without the 'code' column, drop course tables so
# the full schema can be recreated.
STALE_TABLE=$(sqlite3 "${DB_PATH}" "
    SELECT COUNT(*)
    FROM sqlite_master
    WHERE type='table' AND name='courses'
    AND NOT EXISTS (
        SELECT 1 FROM pragma_table_info('courses') WHERE name='code'
    );
")

if [ "${STALE_TABLE}" -gt 0 ]; then
    echo "⚠️  Existing courses table is missing the 'code' column. Recreating course tables..."
    sqlite3 "${DB_PATH}" "
        DROP TABLE IF EXISTS lessons;
        DROP TABLE IF EXISTS modules;
        DROP TABLE IF EXISTS courses;
    "
fi

# Create course schema if it does not exist
sqlite3 "${DB_PATH}" "
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
"

# Seed each course if not already present
seed_course() {
    local code="$1"
    local sql_file="$2"

    local count
    count=$(sqlite3 "${DB_PATH}" "SELECT COUNT(*) FROM courses WHERE code = '${code}';")
    if [ "${count}" -gt 0 ]; then
        echo "⚠️  ${code} already seeded (${count} course found). Skipping."
        return
    fi

    sqlite3 "${DB_PATH}" < "${sql_file}"
    echo "✅ Seeded ${code} successfully."
}

seed_course "MTH1W" "${SCRIPT_DIR}/seed_mth1w.sql"
seed_course "ICD20" "${SCRIPT_DIR}/seed_ics2o.sql"

# Print totals
COURSE_COUNT=$(sqlite3 "${DB_PATH}" "SELECT COUNT(*) FROM courses;")
MODULE_COUNT=$(sqlite3 "${DB_PATH}" "SELECT COUNT(*) FROM modules;")
LESSON_COUNT=$(sqlite3 "${DB_PATH}" "SELECT COUNT(*) FROM lessons;")

echo "📊 Totals in database:"
echo "   Courses:  ${COURSE_COUNT}"
echo "   Modules:  ${MODULE_COUNT}"
echo "   Lessons:  ${LESSON_COUNT}"
