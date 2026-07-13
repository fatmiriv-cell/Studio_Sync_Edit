//! MOTORI I EKSPORTIT: rendëron timeline-in me FFmpeg.
//! Ndërton një filter_complex me trim + setpts (speed) + scale + concat,
//! shton muzikën me fade in/out dhe normalizim loudness (EBU R128),
//! dhe përdor encoder hardware kur mbështetet (NVENC / QSV / AMF).

use crate::autoedit::TimelineClip;
use crate::media::ffmpeg;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    pub output_path: String,
    pub width: u32,       // 1920 / 2560 / 3840 / 7680
    pub height: u32,
    pub fps: u32,         // 24 / 25 / 30 / 50 / 60
    pub encoder: String,  // libx264 / h264_nvenc / hevc_nvenc / prores_ks / ...
    pub quality: u32,     // CRF / CQ (më e ulët = më mirë)
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            output_path: String::new(),
            width: 1920,
            height: 1080,
            fps: 30,
            encoder: "libx264".into(),
            quality: 18,
        }
    }
}

/// Rendëron timeline-in. `media_paths`: media_id → path i skedarit.
/// `music_path`: kënga. `progress` raporton përqindjen e përafërt.
pub fn render(
    clips: &[TimelineClip],
    media_paths: &HashMap<String, String>,
    music_path: &str,
    settings: &ExportSettings,
    mut progress: impl FnMut(u8, &str),
) -> Result<()> {
    if clips.is_empty() {
        return Err(anyhow!("Timeline bosh — s'ka çfarë të eksportohet"));
    }
    progress(5, "Përgatitja e render-it…");

    let mut sorted = clips.to_vec();
    sorted.sort_by(|a, b| a.timeline_start.partial_cmp(&b.timeline_start).unwrap());
    let total_dur: f64 = sorted
        .last()
        .map(|c| c.timeline_start + (c.out_point - c.in_point) / c.speed)
        .unwrap_or(0.0);

    // Inputet: skedarët unikë të videos + muzika në fund.
    let mut input_order: Vec<String> = Vec::new();
    for c in &sorted {
        if !input_order.contains(&c.media_id) {
            input_order.push(c.media_id.clone());
        }
    }
    let music_input_idx = input_order.len();

    let mut args_owned: Vec<String> = Vec::new();
    for id in &input_order {
        let path = media_paths
            .get(id)
            .ok_or_else(|| anyhow!("Media e panjohur në timeline: {id}"))?;
        args_owned.push("-i".into());
        args_owned.push(path.clone());
    }
    args_owned.push("-i".into());
    args_owned.push(music_path.to_string());

    // filter_complex: për çdo klip → trim, speed, scale/pad, fps; pastaj concat.
    let mut fc = String::new();
    let (w, h, fps) = (settings.width, settings.height, settings.fps);
    for (i, c) in sorted.iter().enumerate() {
        let input_idx = input_order.iter().position(|m| m == &c.media_id).unwrap();
        let setpts = if (c.speed - 1.0).abs() > 0.01 {
            format!("setpts=(PTS-STARTPTS)/{:.4},", c.speed)
        } else {
            "setpts=PTS-STARTPTS,".to_string()
        };
        fc.push_str(&format!(
            "[{input_idx}:v]trim=start={:.4}:end={:.4},{setpts}\
             scale={w}:{h}:force_original_aspect_ratio=decrease,\
             pad={w}:{h}:(ow-iw)/2:(oh-ih)/2,fps={fps},format=yuv420p[v{i}];",
            c.in_point, c.out_point
        ));
    }
    for i in 0..sorted.len() {
        fc.push_str(&format!("[v{i}]"));
    }
    fc.push_str(&format!("concat=n={}:v=1:a=0[vout];", sorted.len()));

    // Muzika: prerë në gjatësinë e timeline-it, loudness normalizuar, fade out.
    let fade_start = (total_dur - 1.5).max(0.0);
    fc.push_str(&format!(
        "[{music_input_idx}:a]atrim=start=0:end={total_dur:.3},asetpts=PTS-STARTPTS,\
         loudnorm=I=-14:TP=-1.5:LRA=11,afade=t=in:st=0:d=0.5,\
         afade=t=out:st={fade_start:.3}:d=1.5[aout]"
    ));

    args_owned.push("-filter_complex".into());
    args_owned.push(fc);
    args_owned.extend(["-map".into(), "[vout]".into(), "-map".into(), "[aout]".into()]);

    // Encoder + cilësia.
    args_owned.extend(["-c:v".into(), settings.encoder.clone()]);
    match settings.encoder.as_str() {
        e if e.contains("nvenc") => {
            args_owned.extend(["-preset".into(), "p5".into(), "-cq".into(), settings.quality.to_string()]);
        }
        e if e.contains("qsv") => {
            args_owned.extend(["-global_quality".into(), settings.quality.to_string()]);
        }
        e if e.contains("amf") => {
            args_owned.extend(["-quality".into(), "quality".into(), "-qp_i".into(), settings.quality.to_string()]);
        }
        "prores_ks" => {
            args_owned.extend(["-profile:v".into(), "3".into()]);
        }
        _ => {
            args_owned.extend(["-preset".into(), "medium".into(), "-crf".into(), settings.quality.to_string()]);
        }
    }
    args_owned.extend([
        "-c:a".into(), "aac".into(),
        "-b:a".into(), "256k".into(),
        "-movflags".into(), "+faststart".into(),
        settings.output_path.clone(),
    ]);

    progress(15, "Rendering me FFmpeg…");
    let args: Vec<&str> = args_owned.iter().map(|s| s.as_str()).collect();
    ffmpeg::run_ffmpeg(&args)?;
    progress(100, "Eksporti përfundoi ✓");
    Ok(())
}
