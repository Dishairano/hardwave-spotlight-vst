//! Rust <-> JS protocol for the Spotlight webview.

use serde::{Deserialize, Serialize};

/// Full state packet pushed to the webview at ~60 fps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotlightPacket {
    // ── Analysis data ───────────────────────────────────────────────────────
    /// 4-band energy: [sub, low, mid, high] each 0..1
    pub band_energy: [f32; 4],
    /// Overall RMS energy 0..1
    pub overall_energy: f32,

    /// Transient triggers
    pub kick: bool,
    pub snare: bool,
    pub hihat: bool,

    /// Beat tracking
    pub bpm: f32,
    pub beat_phase: f32,
    pub beat_confidence: f32,

    /// Spectral flux (frame-to-frame change rate)
    pub spectral_flux: f32,

    /// Detected section name
    pub section: String,

    // ── Parameter values ────────────────────────────────────────────────────
    pub energy_param: f32,
    pub strobe_rate: f32,
    pub color_hue: f32,
    pub color_sat: f32,
    pub laser: f32,
    pub sweep: f32,
    pub fog: f32,
    pub led_wall: f32,
    pub scene: String,
    pub camera: String,
    pub venue: String,
    pub ai_enabled: bool,

    // ── Optional spectrum (sent every few frames) ───────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spectrum: Option<Vec<f32>>,
}

impl Default for SpotlightPacket {
    fn default() -> Self {
        Self {
            band_energy: [0.0; 4],
            overall_energy: 0.0,
            kick: false,
            snare: false,
            hihat: false,
            bpm: 0.0,
            beat_phase: 0.0,
            beat_confidence: 0.0,
            spectral_flux: 0.0,
            section: "Intro".into(),
            energy_param: 0.5,
            strobe_rate: 0.0,
            color_hue: 0.0,
            color_sat: 0.8,
            laser: 0.0,
            sweep: 0.5,
            fog: 0.3,
            led_wall: 0.0,
            scene: "Auto".into(),
            camera: "Front".into(),
            venue: "Festival".into(),
            ai_enabled: true,
            spectrum: None,
        }
    }
}

/// JS -> Rust messages from the webview.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum UiMessage {
    #[serde(rename = "set_param")]
    SetParam { id: String, value: f64 },
    #[serde(rename = "set_venue")]
    SetVenue { venue: String },
    #[serde(rename = "set_camera")]
    SetCamera { camera: String },
    #[serde(rename = "export_start")]
    ExportStart {
        width: u32,
        height: u32,
        fps: u32,
    },
    #[serde(rename = "export_cancel")]
    ExportCancel,
    #[serde(rename = "resize")]
    Resize { width: u32, height: u32 },
    #[serde(rename = "save_token")]
    SaveToken { token: String },
    #[serde(rename = "clear_token")]
    ClearToken,
    #[serde(rename = "release_focus")]
    ReleaseFocus,
}
