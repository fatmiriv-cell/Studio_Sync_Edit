//! Vlerësimi i BPM (autokorrelacion) dhe beat tracking me programim dinamik
//! (metoda e Dan Ellis, "Beat Tracking by Dynamic Programming", 2007).

use super::{HOP, SAMPLE_RATE};

const BPM_MIN: f64 = 60.0;
const BPM_MAX: f64 = 200.0;

fn frames_per_sec() -> f64 {
    SAMPLE_RATE as f64 / HOP as f64 // ≈ 86.13 frame/s
}

/// Vlerëson BPM-në globale me autokorrelacion të onset envelope,
/// me peshim log-normal rreth 120 BPM (preferenca njerëzore e tempos).
pub fn estimate_bpm(envelope: &[f32]) -> f64 {
    let fps = frames_per_sec();
    let lag_min = (60.0 / BPM_MAX * fps).round() as usize; // BPM i lartë → lag i vogël
    let lag_max = ((60.0 / BPM_MIN * fps).round() as usize).min(envelope.len() / 2);
    if lag_max <= lag_min {
        return 120.0;
    }

    let mut best_lag = lag_min;
    let mut best_score = f64::MIN;
    for lag in lag_min..=lag_max {
        let mut acf = 0.0f64;
        for i in lag..envelope.len() {
            acf += (envelope[i] * envelope[i - lag]) as f64;
        }
        acf /= (envelope.len() - lag) as f64;

        let bpm = 60.0 * fps / lag as f64;
        // Peshim log-normal rreth 120 BPM.
        let w = (-0.5 * ((bpm / 120.0).ln() / 0.9).powi(2)).exp();
        let score = acf * w;
        if score > best_score {
            best_score = score;
            best_lag = lag;
        }
    }
    let mut bpm = 60.0 * fps / best_lag as f64;
    // Sill në intervalin muzikor të zakonshëm 70–180.
    while bpm < 70.0 {
        bpm *= 2.0;
    }
    while bpm > 180.0 {
        bpm /= 2.0;
    }
    (bpm * 10.0).round() / 10.0
}

/// Beat tracking me programim dinamik: gjen sekuencën e frame-ve që
/// maksimizon (energji onset + rregullsi e tempos).
/// Kthen indekset e frame-ve të beat-eve.
pub fn track_beats(envelope: &[f32], bpm: f64) -> Vec<usize> {
    if envelope.is_empty() {
        return vec![];
    }
    let fps = frames_per_sec();
    let period = 60.0 / bpm * fps; // frame për beat
    let tightness = 400.0f64;

    let n = envelope.len();
    let mut score = vec![0.0f64; n];
    let mut backlink = vec![-1isize; n];

    let win_lo = (period * 0.5).round() as usize;
    let win_hi = (period * 2.0).round() as usize;

    for i in 0..n {
        score[i] = envelope[i] as f64;
        if i < win_lo {
            continue;
        }
        let lo = i.saturating_sub(win_hi);
        let hi = i - win_lo;
        let mut best = f64::MIN;
        let mut best_j = -1isize;
        for j in lo..=hi {
            let interval = (i - j) as f64;
            let penalty = -tightness / 100.0 * (interval / period).ln().powi(2);
            let s = score[j] + penalty;
            if s > best {
                best = s;
                best_j = j as isize;
            }
        }
        if best_j >= 0 && best > 0.0 {
            score[i] += best;
            backlink[i] = best_j;
        }
    }

    // Fillo nga frame-i me score maksimal në fund dhe ndiq zinxhirin mbrapsht.
    let tail_start = n.saturating_sub((period * 1.5) as usize);
    let mut cur = (tail_start..n)
        .max_by(|&a, &b| score[a].partial_cmp(&score[b]).unwrap())
        .unwrap_or(n - 1) as isize;

    let mut beats = Vec::new();
    while cur >= 0 {
        beats.push(cur as usize);
        cur = backlink[cur as usize];
    }
    beats.reverse();
    beats
}
