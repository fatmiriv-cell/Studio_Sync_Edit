//! Onset envelope me spectral flux mbi STFT.
//! Vlerat e larta = goditje (kick, snare, hi-hat, akcente muzikore).

use super::{HOP, WIN};
use rustfft::{num_complex::Complex, FftPlanner};

/// Llogarit onset envelope (spectral flux gjysmë-valor, i normalizuar 0..1).
/// Një vlerë për çdo hop (~11.6 ms).
pub fn onset_envelope(samples: &[f32]) -> Vec<f32> {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(WIN);

    // Dritare Hann e paracaktuar.
    let hann: Vec<f32> = (0..WIN)
        .map(|i| {
            0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / (WIN - 1) as f32).cos()
        })
        .collect();

    let n_frames = if samples.len() > WIN {
        (samples.len() - WIN) / HOP
    } else {
        0
    };
    let n_bins = WIN / 2;

    let mut prev_mag = vec![0.0f32; n_bins];
    let mut flux = Vec::with_capacity(n_frames);
    let mut buf = vec![Complex::new(0.0f32, 0.0f32); WIN];

    for f in 0..n_frames {
        let start = f * HOP;
        for i in 0..WIN {
            buf[i] = Complex::new(samples[start + i] * hann[i], 0.0);
        }
        fft.process(&mut buf);

        // Spectral flux: shuma e rritjeve të magnitudës (gjysmë-valore) —
        // kompresim logaritmik për të theksuar goditjet e ulëta (bass/kick).
        let mut sum = 0.0f32;
        for (i, c) in buf.iter().take(n_bins).enumerate() {
            let mag = (1.0 + c.norm()).ln();
            let d = mag - prev_mag[i];
            if d > 0.0 {
                sum += d;
            }
            prev_mag[i] = mag;
        }
        flux.push(sum);
    }

    normalize_local(&mut flux);
    flux
}

/// Normalizim lokal (heq trendin, mban vetëm pjesën mbi mesataren lokale)
/// që envelope të jetë i qëndrueshëm ndaj ndryshimeve të volumit.
fn normalize_local(env: &mut [f32]) {
    if env.is_empty() {
        return;
    }
    let w = 172; // ~2 s me hop 11.6 ms
    let orig = env.to_vec();
    for i in 0..env.len() {
        let a = i.saturating_sub(w / 2);
        let b = (i + w / 2).min(orig.len());
        let mean = orig[a..b].iter().sum::<f32>() / (b - a).max(1) as f32;
        env[i] = (orig[i] - mean).max(0.0);
    }
    let max = env.iter().cloned().fold(0.0f32, f32::max);
    if max > 0.0 {
        for v in env.iter_mut() {
            *v /= max;
        }
    }
}
