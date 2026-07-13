//! Zbulimi i highlight-eve: gjen momentet më të forta brenda çdo klipi
//! (lëvizje maksimale + mprehtësi + ekspozim i mirë), pa i prerë kufijtë e skenave.

use super::Highlight;

const WIN_SEC: f64 = 2.5; // dritarja bazë e një highlight-i
const MAX_HIGHLIGHTS: usize = 12;

pub fn detect(
    motion: &[f32],
    sharpness: &[f32],
    brightness: &[f32],
    scenes: &[f64],
    fps: f64,
) -> Vec<Highlight> {
    let n = motion.len();
    let win = (WIN_SEC * fps) as usize;
    if n < win || win == 0 {
        return vec![Highlight {
            start: 0.0,
            end: n as f64 / fps,
            score: 0.5,
        }];
    }

    // Score për çdo frame: peshon action-in, fokusimin dhe ekspozimin.
    let frame_score: Vec<f32> = (0..n)
        .map(|i| {
            let exposure = 1.0 - (brightness[i] - 0.5).abs() * 2.0;
            (motion[i] * 10.0).min(1.0) * 0.55 + sharpness[i] * 0.25 + exposure * 0.20
        })
        .collect();

    // Score i dritares = mesatarja e frame score-ve.
    let mut window_scores: Vec<(usize, f32)> = (0..n - win)
        .map(|i| {
            let s = frame_score[i..i + win].iter().sum::<f32>() / win as f32;
            (i, s)
        })
        .collect();
    window_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Zgjidh dritaret më të mira pa mbivendosje dhe pa kapërcim skene.
    let mut picked: Vec<Highlight> = Vec::new();
    for (start_f, score) in window_scores {
        if picked.len() >= MAX_HIGHLIGHTS {
            break;
        }
        let start = start_f as f64 / fps;
        let end = (start_f + win) as f64 / fps;
        let overlaps = picked
            .iter()
            .any(|h| start < h.end + 0.5 && end > h.start - 0.5);
        if overlaps {
            continue;
        }
        // Mos prano dritare që përmban një ndërrim skene në mes.
        let crosses_scene = scenes
            .iter()
            .any(|&s| s > start + 0.2 && s < end - 0.2);
        if crosses_scene {
            continue;
        }
        picked.push(Highlight { start, end, score });
    }

    if picked.is_empty() {
        picked.push(Highlight {
            start: 0.0,
            end: (win as f64 / fps).min(n as f64 / fps),
            score: 0.4,
        });
    }
    picked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    picked
}
