# Almanach

An AI tutor for French classrooms. Inspired by the ancient almanac — a companion that knew the seasons, the tides, and now, the student.

## What It Is

Almanach is a web-based AI learning platform where students log in, enroll in courses, and work through lessons at their own pace. An AI tutor helps them. Teachers watch from a dashboard, see what students are asking about, and can guide the AI's responses if someone is stuck or wandering.

## Core Philosophy

- **Students wander.** If a student starts coding or exploring something creative during a math lesson, that's good. The AI nudges gently: *"C'est cool que tu codes ça ! Tu veux qu'on y revienne dans 10 minutes ?"*
- **Teachers see, they don't police.** The dashboard shows soft insights — not alerts. No orange warnings. No "off-topic" language.
- **The AI is a companion, not a prison guard.**

## What's Here (Post-Gut)

- `orchestrator/` — Rust/Axum server. Auth, chat, courses, lessons, progress tracking
- `static-site/` — Web UI (served by orchestrator)
- `tauri-app/` — Desktop app shell (to be rebranded)
- `ui/` — Yew/WASM UI components (to be repurposed)
- `data/` — SQLite database

## What's Gone

- Agent proxy, container runtime, Docker management, Exo integration
- Agent binaries, premium templates, team system
- Old docs about deployment, security audits, volume mounting

## Getting Started

```bash
cd orchestrator
cargo run
```

The server starts on `http://localhost:3000` (or configured port).

## First Course: Ontario MTH1W

The first curriculum implemented is Ontario Grade 9 Mathematics (MTH1W) — 4 units, 43 lessons, ~110 hours. See `almanach_curriculum_mth1w.md` in the parent workspace for the full plan.

## License

MIT
