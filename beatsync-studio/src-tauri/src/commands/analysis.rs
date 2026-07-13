use super::ProgressEvent;
use crate::{audio, video, AppState};
use tauri::{AppHandle, Emitter, Manager, State};

fn media_path(app: &AppHandle, media_id: &str) -> Result<String, String> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.query_row("SELECT path FROM media WHERE id = ?1", [media_id], |r| {
        r.get(0)
    })
    .map_err(|e| format!("Media nuk u gjet: {e}"))
}

fn store_json(app: &AppHandle, table: &str, media_id: &str, json: &str) -> Result<(), String> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        &format!("INSERT OR REPLACE INTO {table} (media_id, json) VALUES (?1, ?2)"),
        rusqlite::params![media_id, json],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Analiza AI e muzikës: BPM, beats, downbeats, energji, seksione, akcente.
#[tauri::command]
pub async fn analyze_audio(
    app: AppHandle,
    media_id: String,
) -> Result<audio::AudioAnalysis, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = media_path(&app, &media_id)?;
        let emitter = app.clone();
        let mid = media_id.clone();
        let analysis = audio::analyze(&path, move |pct, msg| {
            let _ = emitter.emit(
                "analysis://progress",
                ProgressEvent {
                    job: "audio".into(),
                    target_id: mid.clone(),
                    pct,
                    msg: msg.into(),
                },
            );
        })
        .map_err(|e| e.to_string())?;
        let json = serde_json::to_string(&analysis).map_err(|e| e.to_string())?;
        store_json(&app, "audio_analysis", &media_id, &json)?;
        Ok(analysis)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Analiza AI e videos: skena, lëvizje, cilësi, highlights.
#[tauri::command]
pub async fn analyze_video(
    app: AppHandle,
    media_id: String,
) -> Result<video::VideoAnalysis, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = media_path(&app, &media_id)?;
        let emitter = app.clone();
        let mid = media_id.clone();
        let analysis = video::analyze(&path, move |pct, msg| {
            let _ = emitter.emit(
                "analysis://progress",
                ProgressEvent {
                    job: "video".into(),
                    target_id: mid.clone(),
                    pct,
                    msg: msg.into(),
                },
            );
        })
        .map_err(|e| e.to_string())?;
        let json = serde_json::to_string(&analysis).map_err(|e| e.to_string())?;
        store_json(&app, "video_analysis", &media_id, &json)?;
        Ok(analysis)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Merr analizën e ruajtur (audio ose video) si JSON.
#[tauri::command]
pub fn analysis_get(
    state: State<AppState>,
    media_id: String,
    kind: String,
) -> Result<Option<String>, String> {
    let table = match kind.as_str() {
        "audio" => "audio_analysis",
        "video" => "video_analysis",
        _ => return Err("kind duhet të jetë 'audio' ose 'video'".into()),
    };
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let res: Result<String, _> = conn.query_row(
        &format!("SELECT json FROM {table} WHERE media_id = ?1"),
        [media_id],
        |r| r.get(0),
    );
    match res {
        Ok(j) => Ok(Some(j)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
