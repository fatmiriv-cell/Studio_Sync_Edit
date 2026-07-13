//! Presetet e stileve të editimit. Çdo stil ndryshon ritmin e prerjeve,
//! sjelljen e tranzicioneve, gjatësinë e klipeve dhe ritmin emocional.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditStyle {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Beat-e për prerje në seksione me energji të ulët (verse/intro).
    pub beats_low: u32,
    /// Beat-e për prerje në seksione me energji të lartë (chorus/drop).
    pub beats_high: u32,
    /// Preferenca për prerje vetëm në downbeats (1 = po).
    pub prefer_downbeats: bool,
    /// Tranzicioni i paracaktuar mes klipeve.
    pub transition: String, // cut | crossfade | fade_black
    /// Aktivizon speed ramps në drops.
    pub speed_ramps: bool,
    /// Pesha e lëvizjes vs cilësisë statike kur zgjidhen highlight-et (0..1).
    pub motion_bias: f32,
    /// Raporti i synuar i pamjes (per eksport të sugjeruar).
    pub aspect: String, // 16:9 | 9:16 | 1:1
}

fn s(v: &str) -> String {
    v.to_string()
}

pub fn all_styles() -> Vec<EditStyle> {
    vec![
        EditStyle { id: s("music_video"), name: s("Music Video"), description: s("Prerje agresive në beat, energji e lartë"), beats_low: 4, beats_high: 1, prefer_downbeats: false, transition: s("cut"), speed_ramps: true, motion_bias: 0.8, aspect: s("16:9") },
        EditStyle { id: s("cinematic"), name: s("Cinematic"), description: s("Klipe të gjata, tranzicione të buta, ritëm i qetë"), beats_low: 8, beats_high: 4, prefer_downbeats: true, transition: s("crossfade"), speed_ramps: false, motion_bias: 0.4, aspect: s("16:9") },
        EditStyle { id: s("trailer"), name: s("Movie Trailer"), description: s("Ndërtim gradual, prerje shpërthyese në drops"), beats_low: 8, beats_high: 1, prefer_downbeats: false, transition: s("fade_black"), speed_ramps: true, motion_bias: 0.9, aspect: s("16:9") },
        EditStyle { id: s("commercial"), name: s("Commercial"), description: s("Ritëm i pastër, profesional, 2-4 beat për klip"), beats_low: 4, beats_high: 2, prefer_downbeats: true, transition: s("cut"), speed_ramps: false, motion_bias: 0.6, aspect: s("16:9") },
        EditStyle { id: s("wedding"), name: s("Wedding"), description: s("Emocionale, klipe të gjata, crossfade të buta"), beats_low: 8, beats_high: 4, prefer_downbeats: true, transition: s("crossfade"), speed_ramps: false, motion_bias: 0.3, aspect: s("16:9") },
        EditStyle { id: s("travel"), name: s("Travel"), description: s("Dinamike, plot lëvizje, speed ramps"), beats_low: 4, beats_high: 2, prefer_downbeats: false, transition: s("cut"), speed_ramps: true, motion_bias: 0.7, aspect: s("16:9") },
        EditStyle { id: s("sports"), name: s("Sports"), description: s("Action maksimal, prerje në çdo beat në drops"), beats_low: 2, beats_high: 1, prefer_downbeats: false, transition: s("cut"), speed_ramps: true, motion_bias: 1.0, aspect: s("16:9") },
        EditStyle { id: s("luxury"), name: s("Luxury"), description: s("Elegante, e ngadaltë, çdo 8 beat-e"), beats_low: 8, beats_high: 8, prefer_downbeats: true, transition: s("crossfade"), speed_ramps: false, motion_bias: 0.2, aspect: s("16:9") },
        EditStyle { id: s("fashion"), name: s("Fashion"), description: s("Stilistike, prerje të mprehta në akcente"), beats_low: 2, beats_high: 2, prefer_downbeats: false, transition: s("cut"), speed_ramps: false, motion_bias: 0.5, aspect: s("9:16") },
        EditStyle { id: s("documentary"), name: s("Documentary"), description: s("Natyrore, klipe të gjata, pa efekte"), beats_low: 16, beats_high: 8, prefer_downbeats: true, transition: s("cut"), speed_ramps: false, motion_bias: 0.3, aspect: s("16:9") },
        EditStyle { id: s("youtube"), name: s("YouTube"), description: s("Ritëm i shpejtë që mban vëmendjen"), beats_low: 4, beats_high: 2, prefer_downbeats: false, transition: s("cut"), speed_ramps: false, motion_bias: 0.6, aspect: s("16:9") },
        EditStyle { id: s("tiktok"), name: s("TikTok"), description: s("Vertikale, hiper-e shpejtë, çdo beat"), beats_low: 2, beats_high: 1, prefer_downbeats: false, transition: s("cut"), speed_ramps: true, motion_bias: 0.9, aspect: s("9:16") },
        EditStyle { id: s("reels"), name: s("Instagram Reels"), description: s("Vertikale, moderne, 1-2 beat për klip"), beats_low: 2, beats_high: 1, prefer_downbeats: false, transition: s("cut"), speed_ramps: false, motion_bias: 0.8, aspect: s("9:16") },
    ]
}

pub fn style_by_id(id: &str) -> EditStyle {
    all_styles()
        .into_iter()
        .find(|st| st.id == id)
        .unwrap_or_else(|| all_styles().remove(0))
}
