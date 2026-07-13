//! Moduli i medias: FFmpeg wrapper, probe, thumbnails, dekodim PCM, waveform.
//! FFmpeg thirret si proces i jashtëm (i bashkangjitur me instaluesin ose në PATH).

pub mod ffmpeg;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub duration: f64,
    pub width: i64,
    pub height: i64,
    pub fps: f64,
    pub has_video: bool,
    pub has_audio: bool,
}
