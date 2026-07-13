use crate::media::ffmpeg;
use crate::{db, AppState};
use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Clone, Serialize)]
pub struct MediaItem {
    pub id: String,
    pub project_id: String,
    pub kind: String,
    pub path: String,
    pub name: String,
    pub duration: f64,
    pub width: i64,
    pub height: i64,
    pub fps: f64,
    pub thumbnail: Option<String>,
    pub analyzed: bool,
}

fn row_to_media(r: &rusqlite::Row, analyzed: bool) -> rusqlite::Result<MediaItem> {
    Ok(MediaItem {
        id: r.get(0)?,
        project_id: r.get(1)?,
        kind: r.get(2)?,
        path: r.get(3)?,
        name: r.get(4)?,
        duration: r.get(5)?,
        width: r.get(6)?,
        height: r.get(7)?,
        fps: r.get(8)?,
        thumbnail: r.get(9)?,
        analyzed,
    })
}

/// Importon skedarë media: probe + thumbnail, ruajtje në DB.
/// Jo-destruktive — skedari origjinal vetëm referohet, kurrë nuk kopjohet/preket.
#[tauri::command]
pub async fn media_import(
    app: AppHandle,
    project_id: String,
    paths: Vec<String>,
) -> Result<Vec<MediaItem>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let mut imported = Vec::new();
        for path in paths {
            let info = ffmpeg::probe(&path).map_err(|e| e.to_string())?;
            let kind = if info.has_video { "video" } else { "audio" };
            let id = uuid::Uuid::new_v4().to_string();
            let name = Path::new(&path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());

            let mut thumbnail: Option<String> = None;
            if info.has_video {
                let thumb_path = state.cache_dir.join(format!("{id}.jpg"));
                if ffmpeg::thumbnail(&path, &thumb_path, info.duration * 0.25).is_ok() {
                    thumbnail = Some(thumb_path.to_string_lossy().to_string());
                }
            }

            let conn = state.db.lock().map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT INTO media (id, project_id, kind, path, name, duration, width, height, fps, thumbnail, created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                rusqlite::params![
                    id, project_id, kind, path, name, info.duration, info.width,
                    info.height, info.fps, thumbnail, db::now()
                ],
            )
            .map_err(|e| e.to_string())?;
            drop(conn);

            imported.push(MediaItem {
                id,
                project_id: project_id.clone(),
                kind: kind.into(),
                path,
                name,
                duration: info.duration,
                width: info.width,
                height: info.height,
                fps: info.fps,
                thumbnail,
                analyzed: false,
            });
        }
        Ok(imported)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn media_list(state: State<AppState>, project_id: String) -> Result<Vec<MediaItem>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT m.id, m.project_id, m.kind, m.path, m.name, m.duration, m.width, m.height, m.fps, m.thumbnail,
                    (CASE WHEN aa.media_id IS NOT NULL OR va.media_id IS NOT NULL THEN 1 ELSE 0 END) AS analyzed
             FROM media m
             LEFT JOIN audio_analysis aa ON aa.media_id = m.id
             LEFT JOIN video_analysis va ON va.media_id = m.id
             WHERE m.project_id = ?1 ORDER BY m.created_at",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([project_id], |r| {
            let analyzed: i64 = r.get(10)?;
            row_to_media(r, analyzed == 1)
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn media_remove(state: State<AppState>, media_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM media WHERE id = ?1", [media_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Peaks të waveform-it për vizatimin e audios në timeline.
#[tauri::command]
pub async fn media_waveform(
    app: AppHandle,
    media_id: String,
    bins: usize,
) -> Result<Vec<f32>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let path: String = {
            let conn = state.db.lock().map_err(|e| e.to_string())?;
            conn.query_row(
                "SELECT path FROM media WHERE id = ?1",
                [&media_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?
        };
        ffmpeg::waveform_peaks(&path, bins.min(8000)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
