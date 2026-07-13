//! Shtresa e databazës (SQLite, lokale, e bashkangjitur në build — pa server).
//! Editimi është jo-destruktiv: media origjinale nuk preket kurrë,
//! timeline-i është vetëm referenca (media_id + in/out points).

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS projects (
            id         TEXT PRIMARY KEY,
            name       TEXT NOT NULL,
            style      TEXT NOT NULL DEFAULT 'music_video',
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS media (
            id         TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            kind       TEXT NOT NULL CHECK (kind IN ('audio','video')),
            path       TEXT NOT NULL,
            name       TEXT NOT NULL,
            duration   REAL NOT NULL DEFAULT 0,
            width      INTEGER NOT NULL DEFAULT 0,
            height     INTEGER NOT NULL DEFAULT 0,
            fps        REAL NOT NULL DEFAULT 0,
            thumbnail  TEXT,
            created_at INTEGER NOT NULL
        );

        -- Rezultatet e AI ruhen si JSON (beat map / video analysis).
        CREATE TABLE IF NOT EXISTS audio_analysis (
            media_id TEXT PRIMARY KEY REFERENCES media(id) ON DELETE CASCADE,
            json     TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS video_analysis (
            media_id TEXT PRIMARY KEY REFERENCES media(id) ON DELETE CASCADE,
            json     TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS timeline_clips (
            id             TEXT PRIMARY KEY,
            project_id     TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            track          INTEGER NOT NULL DEFAULT 0,
            media_id       TEXT NOT NULL REFERENCES media(id) ON DELETE CASCADE,
            timeline_start REAL NOT NULL,
            in_point       REAL NOT NULL,
            out_point      REAL NOT NULL,
            speed          REAL NOT NULL DEFAULT 1.0,
            transition     TEXT NOT NULL DEFAULT 'cut'
        );
        CREATE INDEX IF NOT EXISTS idx_clips_project ON timeline_clips(project_id);
        "#,
    )?;
    Ok(conn)
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
