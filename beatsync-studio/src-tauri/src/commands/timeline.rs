use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipRow {
    pub id: String,
    pub project_id: String,
    pub track: i64,
    pub media_id: String,
    pub timeline_start: f64,
    pub in_point: f64,
    pub out_point: f64,
    pub speed: f64,
    pub transition: String,
}

#[tauri::command]
pub fn timeline_get(state: State<AppState>, project_id: String) -> Result<Vec<ClipRow>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, track, media_id, timeline_start, in_point, out_point, speed, transition
             FROM timeline_clips WHERE project_id = ?1 ORDER BY timeline_start",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([project_id], |r| {
            Ok(ClipRow {
                id: r.get(0)?,
                project_id: r.get(1)?,
                track: r.get(2)?,
                media_id: r.get(3)?,
                timeline_start: r.get(4)?,
                in_point: r.get(5)?,
                out_point: r.get(6)?,
                speed: r.get(7)?,
                transition: r.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// Editim jo-destruktiv: përditëson pozicionin/trim/shpejtësinë e një klipi.
#[tauri::command]
pub fn timeline_update_clip(state: State<AppState>, clip: ClipRow) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE timeline_clips SET track=?1, timeline_start=?2, in_point=?3, out_point=?4, speed=?5, transition=?6
         WHERE id = ?7",
        rusqlite::params![
            clip.track, clip.timeline_start, clip.in_point, clip.out_point,
            clip.speed, clip.transition, clip.id
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn timeline_delete_clip(state: State<AppState>, clip_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM timeline_clips WHERE id = ?1", [clip_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn timeline_add_clip(state: State<AppState>, mut clip: ClipRow) -> Result<ClipRow, String> {
    clip.id = uuid::Uuid::new_v4().to_string();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO timeline_clips (id, project_id, track, media_id, timeline_start, in_point, out_point, speed, transition)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        rusqlite::params![
            clip.id, clip.project_id, clip.track, clip.media_id, clip.timeline_start,
            clip.in_point, clip.out_point, clip.speed, clip.transition
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(clip)
}
