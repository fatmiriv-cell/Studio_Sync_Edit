use crate::{db, AppState};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub style: String,
    pub created_at: i64,
}

#[tauri::command]
pub fn project_create(state: State<AppState>, name: String) -> Result<Project, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let created_at = db::now();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO projects (id, name, style, created_at) VALUES (?1, ?2, 'music_video', ?3)",
        rusqlite::params![id, name, created_at],
    )
    .map_err(|e| e.to_string())?;
    Ok(Project {
        id,
        name,
        style: "music_video".into(),
        created_at,
    })
}

#[tauri::command]
pub fn project_list(state: State<AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, style, created_at FROM projects ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Project {
                id: r.get(0)?,
                name: r.get(1)?,
                style: r.get(2)?,
                created_at: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn project_delete(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM projects WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
