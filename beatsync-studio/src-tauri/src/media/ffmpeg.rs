//! Wrapper i FFmpeg/FFprobe. Kërkon binarin në këtë radhë:
//! 1. Ndryshorja e mjedisit `BEATSYNC_FFMPEG_DIR`
//! 2. Dosja `bin/` pranë ekzekutuesit (e paketuar me instaluesin)
//! 3. PATH i sistemit

use super::MediaInfo;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

fn tool(name: &str) -> PathBuf {
    let exe = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    if let Ok(dir) = std::env::var("BEATSYNC_FFMPEG_DIR") {
        let p = Path::new(&dir).join(&exe);
        if p.exists() {
            return p;
        }
    }
    if let Ok(cur) = std::env::current_exe() {
        if let Some(dir) = cur.parent() {
            let p = dir.join("bin").join(&exe);
            if p.exists() {
                return p;
            }
            let p = dir.join(&exe);
            if p.exists() {
                return p;
            }
        }
    }
    PathBuf::from(exe)
}

pub fn ffmpeg() -> PathBuf {
    tool("ffmpeg")
}
pub fn ffprobe() -> PathBuf {
    tool("ffprobe")
}

fn hide_window(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = cmd;
    }
}

pub fn run_ffmpeg(args: &[&str]) -> Result<Vec<u8>> {
    let mut cmd = Command::new(ffmpeg());
    cmd.args(["-hide_banner", "-y"]).args(args);
    hide_window(&mut cmd);
    let out = cmd.output().context("FFmpeg nuk u nis — a është i instaluar?")?;
    if !out.status.success() && out.stdout.is_empty() {
        return Err(anyhow!(
            "FFmpeg dështoi: {}",
            String::from_utf8_lossy(&out.stderr)
                .lines()
                .rev()
                .take(5)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    Ok(out.stdout)
}

/// Merr metadatat e një skedari media me ffprobe (JSON).
pub fn probe(path: &str) -> Result<MediaInfo> {
    let mut cmd = Command::new(ffprobe());
    cmd.args([
        "-v", "error", "-print_format", "json", "-show_format", "-show_streams", path,
    ]);
    hide_window(&mut cmd);
    let out = cmd.output().context("FFprobe nuk u nis")?;
    let v: serde_json::Value = serde_json::from_slice(&out.stdout)
        .context("dalje e pavlefshme nga ffprobe")?;

    let duration = v["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let mut info = MediaInfo {
        duration,
        width: 0,
        height: 0,
        fps: 0.0,
        has_video: false,
        has_audio: false,
    };
    if let Some(streams) = v["streams"].as_array() {
        for s in streams {
            match s["codec_type"].as_str() {
                Some("video") => {
                    info.has_video = true;
                    info.width = s["width"].as_i64().unwrap_or(0);
                    info.height = s["height"].as_i64().unwrap_or(0);
                    if let Some(r) = s["r_frame_rate"].as_str() {
                        let parts: Vec<f64> = r
                            .split('/')
                            .filter_map(|x| x.parse::<f64>().ok())
                            .collect();
                        if parts.len() == 2 && parts[1] > 0.0 {
                            info.fps = parts[0] / parts[1];
                        }
                    }
                }
                Some("audio") => info.has_audio = true,
                _ => {}
            }
        }
    }
    Ok(info)
}

/// Gjeneron një thumbnail JPEG në sekondën `t`.
pub fn thumbnail(path: &str, out_path: &Path, t: f64) -> Result<()> {
    run_ffmpeg(&[
        "-ss", &format!("{t:.2}"),
        "-i", path,
        "-frames:v", "1",
        "-vf", "scale=320:-2",
        "-q:v", "4",
        out_path.to_str().unwrap_or_default(),
    ])?;
    Ok(())
}

/// Dekodon audion në PCM mono f32 @ `sample_rate` Hz (për analizën DSP).
pub fn decode_pcm_mono(path: &str, sample_rate: u32) -> Result<Vec<f32>> {
    let sr = sample_rate.to_string();
    let raw = run_ffmpeg(&[
        "-i", path,
        "-vn",
        "-ac", "1",
        "-ar", &sr,
        "-f", "f32le",
        "-",
    ])?;
    let mut samples = Vec::with_capacity(raw.len() / 4);
    for chunk in raw.chunks_exact(4) {
        samples.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    Ok(samples)
}

/// Dekodon video-n në frame gri të zvogëluara (w x h, 1 bajt/piksel) me `fps` frame/s.
/// Kthen (frames, fps_real). Përdoret nga motori i analizës së videos.
pub fn decode_gray_frames(path: &str, w: u32, h: u32, fps: f64) -> Result<Vec<Vec<u8>>> {
    let vf = format!("fps={fps},scale={w}:{h},format=gray");
    let raw = run_ffmpeg(&["-i", path, "-vf", &vf, "-f", "rawvideo", "-"])?;
    let frame_size = (w * h) as usize;
    Ok(raw
        .chunks_exact(frame_size)
        .map(|c| c.to_vec())
        .collect())
}

/// Gjeneron peaks të waveform-it (max abs për bin) për vizatim në timeline.
pub fn waveform_peaks(path: &str, bins: usize) -> Result<Vec<f32>> {
    let samples = decode_pcm_mono(path, 8000)?;
    if samples.is_empty() || bins == 0 {
        return Ok(vec![]);
    }
    let per_bin = (samples.len() as f64 / bins as f64).max(1.0);
    let mut peaks = Vec::with_capacity(bins);
    for i in 0..bins {
        let start = (i as f64 * per_bin) as usize;
        let end = (((i + 1) as f64 * per_bin) as usize).min(samples.len());
        if start >= end {
            peaks.push(0.0);
            continue;
        }
        let peak = samples[start..end].iter().fold(0.0f32, |m, s| m.max(s.abs()));
        peaks.push(peak.min(1.0));
    }
    Ok(peaks)
}

/// Zbulon encoder-ët hardware në dispozicion (NVENC / QSV / AMF).
pub fn available_encoders() -> Vec<String> {
    let mut cmd = Command::new(ffmpeg());
    cmd.args(["-hide_banner", "-encoders"]);
    hide_window(&mut cmd);
    let Ok(out) = cmd.output() else {
        return vec!["libx264".into()];
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut found = vec!["libx264".to_string(), "libx265".to_string()];
    for enc in [
        "h264_nvenc", "hevc_nvenc",
        "h264_qsv", "hevc_qsv",
        "h264_amf", "hevc_amf",
        "prores_ks",
    ] {
        if text.contains(enc) {
            found.push(enc.to_string());
        }
    }
    found
}
