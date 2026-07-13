use super::ProgressEvent;
use crate::autoedit::TimelineClip;
use crate::export::{self, ExportSettings};
use crate::media::ffmpeg;
use crate::AppState;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, Manager};

/// Zbulon encoder-ët në dispozicion (përfshirë hardware: NVENC/QSV/AMF).
#[tauri::command]
pub fn export_encoders() -> Vec<String> {
    ffmpeg::available_encoders()
}

/// Rendëron timeline-in e projektit në një skedar final.
#[tauri::command]
pub async fn export_render(
    app: AppHandle,
    project_id: String,
    settings: ExportSettings,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();

        // Lexo timeline-in + path-et e medias + këngën.
        let (clips, media_paths, music_path) = {
            let conn = state.db.lock().map_err(|e| e.to_string())?;

            let mut stmt = conn
                .prepare(
                    "SELECT media_id, track, timeline_start, in_point, out_point, speed, transition
                     FROM timeline_clips WHERE project_id = ?1 ORDER BY timeline_start",
                )
                .map_err(|e| e.to_string())?;
            let clips: Vec<TimelineClip> = stmt
                .query_map([&project_id], |r| {
                    Ok(TimelineClip {
                        media_id: r.get(0)?,
                        track: r.get(1)?,
                        timeline_start: r.get(2)?,
                        in_point: r.get(3)?,
                        out_point: r.get(4)?,
                        speed: r.get(5)?,
                        transition: r.get(6)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .filter_map(|c| c.ok())
                .collect();

            let mut stmt = conn
                .prepare("SELECT id, path FROM media WHERE project_id = ?1")
                .map_err(|e| e.to_string())?;
            let media_paths: HashMap<String, String> = stmt
                .query_map([&project_id], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                })
                .map_err(|e| e.to_string())?
                .filter_map(|x| x.ok())
                .collect();

            let music_path: String = conn
                .query_row(
                    "SELECT path FROM media WHERE project_id = ?1 AND kind = 'audio'
                     ORDER BY created_at LIMIT 1",
                    [&project_id],
                    |r| r.get(0),
                )
                .map_err(|_| "Projekti s'ka këngë të importuar".to_string())?;

            (clips, media_paths, music_path)
        };

        let emitter = app.clone();
        let pid = project_id.clone();
        export::render(&clips, &media_paths, &music_path, &settings, move |pct, msg| {
            let _ = emitter.emit(
                "export://progress",
                ProgressEvent {
                    job: "export".into(),
                    target_id: pid.clone(),
                    pct,
                    msg: msg.into(),
                },
            );
        })
        .map_err(|e| e.to_string())?;

        Ok(settings.output_path)
    })
    .await
    .map_err(|e| e.to_string())?
}
