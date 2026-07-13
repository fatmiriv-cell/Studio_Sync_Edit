//! Struktura e këngës: energji për beat, downbeats, seksione
//! (intro/verse/build-up/chorus/drop/outro) dhe beat-et e theksuara.

use super::{Section, SAMPLE_RATE};

/// Energjia RMS midis çdo çifti beat-esh, e normalizuar 0..1.
pub fn energy_per_beat(samples: &[f32], beat_times: &[f64]) -> Vec<f32> {
    let sr = SAMPLE_RATE as f64;
    let mut energies = Vec::with_capacity(beat_times.len());
    for (i, &t) in beat_times.iter().enumerate() {
        let start = (t * sr) as usize;
        let end_t = beat_times.get(i + 1).copied().unwrap_or(t + 0.5);
        let end = ((end_t * sr) as usize).min(samples.len());
        if start >= end {
            energies.push(0.0);
            continue;
        }
        let rms = (samples[start..end].iter().map(|s| s * s).sum::<f32>()
            / (end - start) as f32)
            .sqrt();
        energies.push(rms);
    }
    let max = energies.iter().cloned().fold(0.0f32, f32::max);
    if max > 0.0 {
        for e in energies.iter_mut() {
            *e /= max;
        }
    }
    energies
}

/// Downbeats: zgjedh fazën (0..3) ku shuma e energjisë onset është maksimale,
/// pastaj merr çdo beat të 4-t nga ajo fazë.
pub fn downbeats(envelope: &[f32], beat_frames: &[usize], beat_times: &[f64]) -> Vec<f64> {
    if beat_frames.len() < 8 {
        return beat_times.to_vec();
    }
    let mut best_phase = 0;
    let mut best_sum = f32::MIN;
    for phase in 0..4 {
        let sum: f32 = beat_frames
            .iter()
            .skip(phase)
            .step_by(4)
            .map(|&f| envelope.get(f).copied().unwrap_or(0.0))
            .sum();
        if sum > best_sum {
            best_sum = sum;
            best_phase = phase;
        }
    }
    beat_times
        .iter()
        .skip(best_phase)
        .step_by(4)
        .copied()
        .collect()
}

/// Beat-et me onset dukshëm mbi mesataren — kick/snare të fortë, akcente, drops.
/// Kandidatët kryesorë ku motori auto-edit vendos prerjet.
pub fn accent_beats(envelope: &[f32], beat_frames: &[usize], beat_times: &[f64]) -> Vec<f64> {
    let strengths: Vec<f32> = beat_frames
        .iter()
        .map(|&f| {
            // Maksimumi në ±3 frame rreth beat-it.
            let a = f.saturating_sub(3);
            let b = (f + 3).min(envelope.len());
            envelope[a..b].iter().cloned().fold(0.0f32, f32::max)
        })
        .collect();
    if strengths.is_empty() {
        return vec![];
    }
    let mean = strengths.iter().sum::<f32>() / strengths.len() as f32;
    beat_times
        .iter()
        .zip(&strengths)
        .filter(|(_, &s)| s > mean * 1.4)
        .map(|(&t, _)| t)
        .collect()
}

/// Segmentim strukturor i bazuar në kurbën e energjisë (mesatare rrëshqitëse
/// mbi 8 beat-e): pjesët me energji të lartë = chorus/drop, rritjet = build-up,
/// të ulëtat = verse; fillimi/fundi = intro/outro.
pub fn detect_sections(beat_times: &[f64], energy: &[f32], duration: f64) -> Vec<Section> {
    if beat_times.len() < 16 {
        return vec![Section {
            label: "song".into(),
            start: 0.0,
            end: duration,
            energy: 0.5,
        }];
    }

    // Mesatare rrëshqitëse mbi 8 beat-e.
    let w = 8usize;
    let smooth: Vec<f32> = (0..energy.len())
        .map(|i| {
            let a = i.saturating_sub(w / 2);
            let b = (i + w / 2).min(energy.len());
            energy[a..b].iter().sum::<f32>() / (b - a).max(1) as f32
        })
        .collect();

    let mean = smooth.iter().sum::<f32>() / smooth.len() as f32;
    let high = mean * 1.15;
    let low = mean * 0.75;

    // Klasifiko çdo beat, pastaj bashko rendet e njëpasnjëshme në seksione.
    #[derive(PartialEq, Clone, Copy)]
    enum K {
        Low,
        Mid,
        High,
    }
    let classes: Vec<K> = smooth
        .iter()
        .map(|&e| {
            if e >= high {
                K::High
            } else if e <= low {
                K::Low
            } else {
                K::Mid
            }
        })
        .collect();

    let mut sections: Vec<Section> = Vec::new();
    let mut seg_start = 0usize;
    for i in 1..=classes.len() {
        let boundary = i == classes.len() || classes[i] != classes[seg_start];
        // Mos lejo seksione më të shkurtra se 8 beat-e (bashkoji me para-ardhësin).
        if boundary && i - seg_start >= 8 {
            let start_t = beat_times[seg_start];
            let end_t = if i < beat_times.len() {
                beat_times[i]
            } else {
                duration
            };
            let e = smooth[seg_start..i].iter().sum::<f32>() / (i - seg_start) as f32;
            let label = match classes[seg_start] {
                K::High => "chorus",
                K::Mid => "build_up",
                K::Low => "verse",
            };
            sections.push(Section {
                label: label.into(),
                start: start_t,
                end: end_t,
                energy: e,
            });
            seg_start = i;
        } else if boundary {
            // Segment shumë i shkurtër — vazhdo pa e mbyllur.
            if i < classes.len() {
                continue;
            }
            if let Some(last) = sections.last_mut() {
                last.end = duration;
            }
        }
    }
    if sections.is_empty() {
        sections.push(Section {
            label: "song".into(),
            start: 0.0,
            end: duration,
            energy: mean,
        });
    }

    // Etiketimi kontekstual: i pari = intro, i fundit = outro,
    // seksioni i parë "chorus" pas një build-up = drop.
    if let Some(first) = sections.first_mut() {
        if first.label == "verse" {
            first.label = "intro".into();
        }
    }
    let n = sections.len();
    if n > 1 {
        if sections[n - 1].label == "verse" {
            sections[n - 1].label = "outro".into();
        }
        for i in 1..n {
            if sections[i].label == "chorus" && sections[i - 1].label == "build_up" {
                sections[i].label = "drop".into();
            }
        }
    }
    sections
}
