//! MOTORI AI I VIDEOS — 100% lokal.
//!
//! Video dekodohet nga FFmpeg në frame gri të zvogëluara (64×36 @ 6 fps) dhe
//! analizohet në Rust: kufij skenash, intensitet lëvizjeje, ndriçim, mprehtësi,
//! dridhje kamere, score cilësie dhe zbulim highlight-esh.
//! Zbulimi i fytyrave/emocioneve shtohet përmes modulit `ai/` (ONNX).

pub mod highlights;

use crate::media::ffmpeg;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const ANALYSIS_FPS: f64 = 6.0;
const W: u32 = 64;
const H: u32 = 36;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub start: f64,
    pub end: f64,
    pub score: f32, // 0..1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoAnalysis {
    pub duration: f64,
    /// Kufijtë e skenave (sekonda).
    pub scenes: Vec<f64>,
    /// Kurba e intensitetit të lëvizjes (një vlerë për frame analize, 0..1).
    pub motion: Vec<f32>,
    pub analysis_fps: f64,
    pub avg_brightness: f32, // 0..1
    pub avg_sharpness: f32,  // 0..1
    pub shake: f32,          // 0..1 (dridhje kamere)
    /// Score i përgjithshëm i cilësisë së klipit (0..100).
    pub quality_score: f32,
    /// Momentet më të forta të klipit, të renditura sipas score.
    pub highlights: Vec<Highlight>,
}

pub fn analyze(path: &str, mut progress: impl FnMut(u8, &str)) -> Result<VideoAnalysis> {
    progress(10, "Dekodimi i frame-ve…");
    let frames = ffmpeg::decode_gray_frames(path, W, H, ANALYSIS_FPS)?;
    if frames.len() < 4 {
        return Err(anyhow!("Video shumë e shkurtër për analizë"));
    }
    let duration = frames.len() as f64 / ANALYSIS_FPS;

    progress(45, "Lëvizja, skenat, ndriçimi…");
    let px = (W * H) as f32;

    // Ndriçimi mesatar për frame (0..1).
    let brightness: Vec<f32> = frames
        .iter()
        .map(|f| f.iter().map(|&b| b as f32).sum::<f32>() / px / 255.0)
        .collect();

    // Mprehtësia: mesatarja e gradientit horizontal (proxy i thjeshtë i fokusit).
    let sharpness: Vec<f32> = frames
        .iter()
        .map(|f| {
            let mut g = 0.0f32;
            for row in 0..H as usize {
                let base = row * W as usize;
                for col in 1..W as usize {
                    g += (f[base + col] as f32 - f[base + col - 1] as f32).abs();
                }
            }
            (g / px / 40.0).min(1.0)
        })
        .collect();

    // Lëvizja: diferenca mesatare absolute mes frame-ve fqinje.
    let mut motion = vec![0.0f32];
    for i in 1..frames.len() {
        let d: f32 = frames[i]
            .iter()
            .zip(&frames[i - 1])
            .map(|(&a, &b)| (a as f32 - b as f32).abs())
            .sum::<f32>()
            / px
            / 255.0;
        motion.push(d);
    }

    progress(70, "Kufijtë e skenave dhe dridhja…");
    // Skena = kërcim i madh i diferencës krahasuar me fqinjët.
    let mut scenes = vec![0.0f64];
    for i in 2..motion.len() {
        let local = (motion[i - 1] + motion[i.saturating_sub(2)]) / 2.0 + 0.02;
        if motion[i] > 0.14 && motion[i] > local * 3.5 {
            let t = i as f64 / ANALYSIS_FPS;
            if t - scenes.last().unwrap() > 0.5 {
                scenes.push(t);
            }
        }
    }

    // Dridhja e kamerës: variacion i shpejtë por i vogël i lëvizjes.
    let shake = {
        let mut var = 0.0f32;
        for i in 1..motion.len() {
            var += (motion[i] - motion[i - 1]).abs();
        }
        (var / motion.len() as f32 * 25.0).min(1.0)
    };

    let avg_brightness = brightness.iter().sum::<f32>() / brightness.len() as f32;
    let avg_sharpness = sharpness.iter().sum::<f32>() / sharpness.len() as f32;
    let avg_motion = motion.iter().sum::<f32>() / motion.len() as f32;

    // Score cilësie: ekspozim i mirë + mprehtësi + lëvizje e gjallë - dridhje.
    let exposure_score = 1.0 - (avg_brightness - 0.5).abs() * 2.0;
    let quality_score = (exposure_score * 30.0
        + avg_sharpness * 30.0
        + (avg_motion * 8.0).min(1.0) * 25.0
        + (1.0 - shake) * 15.0)
        .clamp(0.0, 100.0);

    progress(90, "Zbulimi i highlight-eve…");
    let highlights =
        highlights::detect(&motion, &sharpness, &brightness, &scenes, ANALYSIS_FPS);

    progress(100, "Analiza e videos ✓");
    Ok(VideoAnalysis {
        duration,
        scenes,
        motion,
        analysis_fps: ANALYSIS_FPS,
        avg_brightness,
        avg_sharpness,
        shake,
        quality_score,
        highlights,
    })
}
