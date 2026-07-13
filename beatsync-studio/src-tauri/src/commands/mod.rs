//! Komandat IPC — API publike e backend-it, e thirrur nga React përmes
//! `invoke()`. Operacionet e gjata ekzekutohen në thread-e blocking dhe
//! raportojnë progres me event-e (`analysis://progress`, `export://progress`).

pub mod analysis;
pub mod autoedit_cmd;
pub mod export_cmd;
pub mod media_cmd;
pub mod project;
pub mod timeline;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub job: String,
    pub target_id: String,
    pub pct: u8,
    pub msg: String,
}
