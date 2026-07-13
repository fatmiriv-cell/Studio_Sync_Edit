//! Zemra e auto-edit: gjeneron cut-list ku çdo prerje bie mbi beat.
//!
//! Algoritmi:
//! 1. Ecën nëpër beat-et e këngës seksion pas seksioni.
//! 2. Gjatësia e çdo klipi (në beat-e) varet nga energjia e seksionit + stili.
//! 3. Për çdo slot zgjedh highlight-in më të mirë të papërdorur
//!    (score × bias i lëvizjes × diversitet klipesh — jo dy herë rresht i njëjti klip).
//! 4. Në drops aplikon speed ramps nëse stili i lejon.

use crate::audio::AudioAnalysis;
use crate::video::VideoAnalysis;
use crate::autoedit::styles::EditStyle;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineClip {
    pub media_id: String,
    pub track: i64,
    pub timeline_start: f64,
    pub in_point: f64,
    pub out_point: f64,
    pub speed: f64,
    pub transition: String,
}

struct Candidate {
    media_id: String,
    start: f64,
    end: f64,
    score: f32,
    used: bool,
}

pub fn run(
    audio: &AudioAnalysis,
    videos: &[(String, VideoAnalysis)],
    style: &EditStyle,
) -> Vec<TimelineClip> {
    // Mblidh të gjithë highlight-et si kandidatë.
    let mut candidates: Vec<Candidate> = Vec::new();
    for (media_id, va) in videos {
        let motion_avg: f32 =
            va.motion.iter().sum::<f32>() / va.motion.len().max(1) as f32;
        for h in &va.highlights {
            // Score i kombinuar: highlight + cilësia e klipit + bias i lëvizjes.
            let s = h.score * 0.6
                + (va.quality_score / 100.0) * 0.25
                + (motion_avg * 10.0).min(1.0) * 0.15 * style.motion_bias;
            candidates.push(Candidate {
                media_id: media_id.clone(),
                start: h.start,
                end: h.end,
                score: s,
                used: false,
            });
        }
    }
    if candidates.is_empty() || audio.beats.len() < 4 {
        return vec![];
    }

    let beats = &audio.beats;
    let mut clips: Vec<TimelineClip> = Vec::new();
    let mut beat_idx = 0usize;
    let mut last_media: String = String::new();

    while beat_idx < beats.len() - 1 {
        let t = beats[beat_idx];

        // Gjej seksionin aktual dhe energjinë e tij.
        let section = audio
            .sections
            .iter()
            .find(|s| t >= s.start && t < s.end);
        let (energy, label) = section
            .map(|s| (s.energy, s.label.as_str()))
            .unwrap_or((0.5, "song"));

        let is_high = matches!(label, "chorus" | "drop") || energy > 0.6;
        let mut beats_per_cut = if is_high {
            style.beats_high
        } else {
            style.beats_low
        } as usize;

        // Në downbeat-preferencë, zgjat deri në downbeat-in tjetër.
        if style.prefer_downbeats && !audio.downbeats.is_empty() {
            let next_down = audio
                .downbeats
                .iter()
                .find(|&&d| d > t + 0.05)
                .copied();
            if let Some(d) = next_down {
                let n = beats[beat_idx..]
                    .iter()
                    .take_while(|&&b| b < d + 0.05)
                    .count();
                if n > 0 {
                    beats_per_cut = beats_per_cut.max(n);
                }
            }
        }

        let end_idx = (beat_idx + beats_per_cut).min(beats.len() - 1);
        let slot_dur = beats[end_idx] - t;
        if slot_dur < 0.15 {
            beat_idx = end_idx.max(beat_idx + 1);
            continue;
        }

        // Speed ramp: në drops luaj klipin me 0.5x për gjysmën e parë (slow-mo).
        let speed = if style.speed_ramps && label == "drop" && beat_idx % 8 < 2 {
            0.5
        } else {
            1.0
        };
        let source_needed = slot_dur * speed;

        // Zgjidh kandidatin më të mirë të papërdorur, me diversitet klipesh.
        let mut best: Option<usize> = None;
        let mut best_score = f32::MIN;
        for (i, c) in candidates.iter().enumerate() {
            if c.used || (c.end - c.start) < source_needed {
                continue;
            }
            let diversity = if c.media_id == last_media { 0.7 } else { 1.0 };
            let sc = c.score * diversity;
            if sc > best_score {
                best_score = sc;
                best = Some(i);
            }
        }
        // Nëse të gjithë u përdorën, rikthe përdorimin (kënga më e gjatë se materiali).
        let idx = match best {
            Some(i) => i,
            None => {
                for c in candidates.iter_mut() {
                    c.used = false;
                }
                match candidates
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.end - c.start >= source_needed)
                    .max_by(|a, b| a.1.score.partial_cmp(&b.1.score).unwrap())
                {
                    Some((i, _)) => i,
                    None => break,
                }
            }
        };

        let c = &mut candidates[idx];
        c.used = true;
        last_media = c.media_id.clone();

        // Merr pjesën qendrore të highlight-it.
        let mid = (c.start + c.end) / 2.0;
        let in_point = (mid - source_needed / 2.0).max(c.start);
        let out_point = in_point + source_needed;

        clips.push(TimelineClip {
            media_id: c.media_id.clone(),
            track: 0,
            timeline_start: t,
            in_point,
            out_point,
            speed,
            transition: if clips.is_empty() {
                "cut".to_string()
            } else {
                style.transition.clone()
            },
        });

        beat_idx = end_idx;
    }

    clips
}
