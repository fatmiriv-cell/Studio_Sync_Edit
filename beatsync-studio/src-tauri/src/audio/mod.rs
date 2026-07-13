//! MOTORI AI I MUZIKËS — 100% lokal, DSP e pastër në Rust.
//!
//! Pipeline: PCM mono → STFT → spectral flux (onset envelope) →
//! BPM (autokorrelacion) → beat tracking (programim dinamik, metoda Ellis) →
//! downbeats (faza me energji maksimale) → energji + seksione + drops.

pub mod onset;
pub mod structure;
pub mod tempo;

use crate::media::ffmpeg;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const SAMPLE_RATE: u32 = 44100;
pub const HOP: usize = 512; // ~11.6 ms rezolucion kohor
pub const WIN: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub label: String, // intro / verse / build_up / chorus / drop / bridge / outro
    pub start: f64,
    pub end: f64,
    pub energy: f32, // 0..1
}

/// Beat map i plotë i këngës — produkti kryesor i analizës.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAnalysis {
    pub duration: f64,
    pub bpm: f64,
    /// Pozicionet e çdo beat-i (sekonda).
    pub beats: Vec<f64>,
    /// Çdo beat i parë i masës (1 në 4).
    pub downbeats: Vec<f64>,
    /// Kurba e energjisë RMS, një vlerë për beat (0..1).
    pub energy_per_beat: Vec<f32>,
    /// Seksionet strukturore të këngës.
    pub sections: Vec<Section>,
    /// Beat-et më të theksuara (kick/snare të fortë, drops) — kandidatë për prerje.
    pub accent_beats: Vec<f64>,
}

/// Analizon një skedar muzike dhe kthen beat map-in e plotë.
/// `progress` thirret me (pct 0..100, mesazh).
pub fn analyze(path: &str, mut progress: impl FnMut(u8, &str)) -> Result<AudioAnalysis> {
    progress(5, "Dekodimi i audios…");
    let samples = ffmpeg::decode_pcm_mono(path, SAMPLE_RATE)?;
    if samples.len() < SAMPLE_RATE as usize {
        return Err(anyhow!("Audio shumë e shkurtër për analizë"));
    }
    let duration = samples.len() as f64 / SAMPLE_RATE as f64;

    progress(25, "Zbulimi i onset-eve (spectral flux)…");
    let envelope = onset::onset_envelope(&samples);

    progress(50, "Vlerësimi i BPM…");
    let bpm = tempo::estimate_bpm(&envelope);

    progress(65, "Beat tracking…");
    let beats = tempo::track_beats(&envelope, bpm);
    let beat_times: Vec<f64> = beats
        .iter()
        .map(|&f| f as f64 * HOP as f64 / SAMPLE_RATE as f64)
        .collect();

    progress(80, "Struktura e këngës: energji, seksione, drops…");
    let energy_per_beat = structure::energy_per_beat(&samples, &beat_times);
    let downbeats = structure::downbeats(&envelope, &beats, &beat_times);
    let sections = structure::detect_sections(&beat_times, &energy_per_beat, duration);
    let accent_beats = structure::accent_beats(&envelope, &beats, &beat_times);

    progress(100, "Beat map i plotë ✓");
    Ok(AudioAnalysis {
        duration,
        bpm,
        beats: beat_times,
        downbeats,
        energy_per_beat,
        sections,
        accent_beats,
    })
}
