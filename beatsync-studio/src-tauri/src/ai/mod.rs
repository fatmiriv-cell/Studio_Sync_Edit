//! Shtresa e AI me ONNX Runtime — e izoluar dhe e gatshme për zgjerim.
//!
//! Analiza bazë (beat map, lëvizje, skena, highlights) bëhet me DSP/CV
//! në modulet `audio/` dhe `video/` pa asnjë model të jashtëm — punon kudo.
//!
//! Ky modul shton modele opsionale ONNX (fytyra, emocione, objekte) që
//! ekzekutohen 100% lokalisht me CPU (fallback) ose GPU (DirectML/CUDA).
//!
//! SI TË SHTOSH NJË MODEL:
//! 1. Shto `ort = { version = "2", features = ["directml"] }` në Cargo.toml.
//! 2. Vendos skedarin `.onnx` në dosjen `models/` pranë ekzekutuesit
//!    (p.sh. `models/yolov8n-face.onnx` — shkarkohet një herë, punon offline).
//! 3. Implemento trait-in `AiModel` më poshtë dhe regjistroje në `registry()`.
//! 4. Thirre nga `video::analyze` për të pasuruar score-in e highlight-eve.
//!
//! Asnjë pjesë tjetër e kodit nuk ndryshon — kontrata mbetet e njëjtë.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub label: String, // face / person / animal / vehicle / building / …
    pub confidence: f32,
    /// Kutia normalizuese 0..1: (x, y, w, h)
    pub bbox: (f32, f32, f32, f32),
}

/// Kontrata e çdo modeli AI lokal.
pub trait AiModel: Send + Sync {
    fn id(&self) -> &'static str;
    /// Analizon një frame RGB (w, h, të dhëna) dhe kthen detektimet.
    fn infer(&self, width: u32, height: u32, rgb: &[u8]) -> anyhow::Result<Vec<Detection>>;
}

/// Regjistri i modeleve të ngarkuara. Bosh në MVP — mbushet kur
/// shtohen modele ONNX sipas udhëzimeve më lart.
pub fn registry() -> Vec<Box<dyn AiModel>> {
    Vec::new()
}
