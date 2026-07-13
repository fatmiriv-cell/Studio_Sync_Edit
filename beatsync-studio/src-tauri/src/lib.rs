//! AI BeatSync Studio — backend entrypoint.
//!
//! Arkitektura modulare: çdo modul është i izoluar dhe komunikon vetëm
//! përmes API-ve publike. UI (React) flet me backend-in vetëm përmes
//! komandave në `commands/`.

pub mod ai;
pub mod audio;
pub mod autoedit;
pub mod commands;
pub mod db;
pub mod export;
pub mod media;
pub mod video;

use std::sync::Mutex;

/// Gjendja globale e aplikacionit (e menaxhuar nga Tauri).
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    /// Dosja e cache-it (thumbnails, waveforms, render të përkohshëm).
    pub cache_dir: std::path::PathBuf,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::Manager;
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("nuk u gjet app data dir");
            std::fs::create_dir_all(&data_dir).expect("krijimi i data dir dështoi");
            let cache_dir = data_dir.join("cache");
            std::fs::create_dir_all(&cache_dir).expect("krijimi i cache dir dështoi");

            let conn = db::open(&data_dir.join("beatsync.db"))
                .expect("hapja e databazës dështoi");
            app.manage(AppState {
                db: Mutex::new(conn),
                cache_dir,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project::project_create,
            commands::project::project_list,
            commands::project::project_delete,
            commands::media_cmd::media_import,
            commands::media_cmd::media_list,
            commands::media_cmd::media_remove,
            commands::media_cmd::media_waveform,
            commands::analysis::analyze_audio,
            commands::analysis::analyze_video,
            commands::analysis::analysis_get,
            commands::autoedit_cmd::autoedit_styles,
            commands::autoedit_cmd::autoedit_run,
            commands::timeline::timeline_get,
            commands::timeline::timeline_update_clip,
            commands::timeline::timeline_delete_clip,
            commands::timeline::timeline_add_clip,
            commands::export_cmd::export_encoders,
            commands::export_cmd::export_render,
        ])
        .run(tauri::generate_context!())
        .expect("gabim gjatë nisjes së aplikacionit");
}
