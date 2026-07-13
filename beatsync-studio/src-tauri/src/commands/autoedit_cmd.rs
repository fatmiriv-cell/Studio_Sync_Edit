use crate::autoedit::{self, styles};
use crate::commands::timeline::ClipRow;
use crate::{audio, video, AppState};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn autoedit_styles() -> Vec<styles::EditStyle> {
    styles::all_styles()
}

/// Ekzekuton auto-edit: lexon beat map-in + analizat e videove nga DB,
/// gjeneron cut-list sinkron me beat dhe e shkruan si timeline në DB.
/// Kthen klipet e krijuara.
#[tauri::command]
pub async fn autoedit_run(
    app: AppHandle,
    project_id: String,
    style_id: String,
) -> Result<Vec<ClipRow>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let style = styles::style_by_id(&style_id);

        // Lexo analizën e muzikës (kënga e parë e analizuar e projektit).
        let (audio_analysis, videos): (audio::AudioAnalysis, Vec<(String, video::VideoAnalysis)>) = {
            let conn = state.db.lock().map_err(|e| e.to_string())?;

            let audio_json: String = conn
                .query_row(
                    "SELECT aa.json FROM audio_analysis aa
                     JOIN media m ON m.id = aa.media_id
                     WHERE m.project_id = ?1 AND m.kind = 'audio'
                     ORDER BY m.created_at LIMIT 1",
                    [&project_id],
                    |r| r.get(0),
                )
                .map_err(|_| "Asnjë këngë e analizuar — importo dhe analizo muzikën së pari".to_string())?;
            let audio_analysis: audio::AudioAnalysis =
                serde_json::from_str(&audio_json).map_err(|e| e.to_string())?;

            let mut stmt = conn
                .prepare(
                    "SELECT va.media_id, va.json FROM video_analysis va
                     JOIN media m ON m.id = va.media_id
                     WHERE m.project_id = ?1 ORDER BY m.created_at",
                )
                .map_err(|e| e.to_string())?;
            let videos: Vec<(String, video::VideoAnalysis)> = stmt
                .query_map([&project_id], |r| {
                    let id: String = r.get(0)?;
                    let json: String = r.get(1)?;
                    Ok((id, json))
                })
                .map_err(|e| e.to_string())?
                .filter_map(|row| row.ok())
                .filter_map(|(id, json)| {
                    serde_json::from_str::<video::VideoAnalysis>(&json)
                        .ok()
                        .map(|va| (id, va))
                })
                .collect();
            (audio_analysis, videos)
        };

        if videos.is_empty() {
            return Err("Asnjë video e analizuar — importo dhe analizo klipet së pari".into());
        }

        // Gjenero cut-list.
        let clips = autoedit::run(&audio_analysis, &videos, &style);
        if clips.is_empty() {
            return Err("Auto-edit nuk gjeneroi asnjë klip — kontrollo materialin".into());
        }

        // Shkruaj timeline-in në DB (zëvendëso timeline-in ekzistues).
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM timeline_clips WHERE project_id = ?1",
            [&project_id],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE projects SET style = ?1 WHERE id = ?2",
            rusqlite::params![style.id, project_id],
        )
        .map_err(|e| e.to_string())?;

        let mut rows = Vec::with_capacity(clips.len());
        for c in &clips {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO timeline_clips (id, project_id, track, media_id, timeline_start, in_point, out_point, speed, transition)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                rusqlite::params![
                    id, project_id, c.track, c.media_id, c.timeline_start,
                    c.in_point, c.out_point, c.speed, c.transition
                ],
            )
            .map_err(|e| e.to_string())?;
            rows.push(ClipRow {
                id,
                project_id: project_id.clone(),
                track: c.track,
                media_id: c.media_id.clone(),
                timeline_start: c.timeline_start,
                in_point: c.in_point,
                out_point: c.out_point,
                speed: c.speed,
                transition: c.transition.clone(),
            });
        }
        Ok(rows)
    })
    .await
    .map_err(|e| e.to_string())?
}
