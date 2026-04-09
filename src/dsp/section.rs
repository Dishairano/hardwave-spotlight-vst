//! Section classifier state machine.
//!
//! Detects musical sections (Intro, Build, Drop, Breakdown, Sustain, Outro)
//! based on energy levels, spectral flux, and their derivatives over time.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Intro,
    Build,
    Drop,
    Breakdown,
    Sustain,
    Outro,
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Section::Intro => write!(f, "Intro"),
            Section::Build => write!(f, "Build"),
            Section::Drop => write!(f, "Drop"),
            Section::Breakdown => write!(f, "Breakdown"),
            Section::Sustain => write!(f, "Sustain"),
            Section::Outro => write!(f, "Outro"),
        }
    }
}

/// Minimum number of FFT frames before a section transition is allowed.
/// At 44.1kHz with 2048-pt FFT, each frame ≈ 46ms. 2 bars at 150 BPM ≈ 3.2s ≈ 69 frames.
const MIN_FRAMES_BEFORE_TRANSITION: u32 = 60;

pub struct SectionClassifier {
    sample_rate: f32,
    current: Section,
    frames_in_section: u32,

    /// Running average of energy (for detecting relative changes).
    energy_avg: f32,
    /// Slope of energy over recent frames.
    energy_slope: f32,
    prev_energy: f32,

    /// History for slope calculation.
    energy_history: Vec<f32>,
    history_pos: usize,
}

const SLOPE_WINDOW: usize = 32; // ~1.5 seconds of frames

impl SectionClassifier {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            current: Section::Intro,
            frames_in_section: 0,
            energy_avg: 0.0,
            energy_slope: 0.0,
            prev_energy: 0.0,
            energy_history: vec![0.0; SLOPE_WINDOW],
            history_pos: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
    }

    pub fn reset(&mut self) {
        self.current = Section::Intro;
        self.frames_in_section = 0;
        self.energy_avg = 0.0;
        self.energy_slope = 0.0;
        self.prev_energy = 0.0;
        self.energy_history.iter_mut().for_each(|s| *s = 0.0);
        self.history_pos = 0;
    }

    /// Process one analysis frame.
    /// - `energy`: overall energy 0..1
    /// - `flux`: spectral flux
    /// - `flux_deriv`: rate of change of flux
    /// - `bpm`: current detected BPM (for bar-length calculations)
    pub fn process(&mut self, energy: f32, flux: f32, flux_deriv: f32, _bpm: f32) {
        self.frames_in_section += 1;

        // Update running average.
        let alpha = 0.05;
        self.energy_avg = self.energy_avg + alpha * (energy - self.energy_avg);

        // Store in history and compute slope.
        let oldest = self.energy_history[self.history_pos];
        self.energy_history[self.history_pos] = energy;
        self.history_pos = (self.history_pos + 1) % SLOPE_WINDOW;
        self.energy_slope = (energy - oldest) / SLOPE_WINDOW as f32;

        self.prev_energy = energy;

        // Only allow transitions after minimum time in current section.
        if self.frames_in_section < MIN_FRAMES_BEFORE_TRANSITION {
            return;
        }

        let next = self.evaluate_transition(energy, flux, flux_deriv);
        if next != self.current {
            self.current = next;
            self.frames_in_section = 0;
        }
    }

    fn evaluate_transition(&self, energy: f32, flux: f32, flux_deriv: f32) -> Section {
        // Intro: very low energy and flux.
        if energy < 0.15 && flux < 0.1 {
            return Section::Intro;
        }

        // Drop: sudden energy spike well above recent average.
        if energy > self.energy_avg + 0.4 && energy > 0.5 {
            return Section::Drop;
        }

        // Breakdown: energy drops significantly below recent average.
        if energy < self.energy_avg - 0.3 && energy < 0.35 && self.current != Section::Intro {
            return Section::Breakdown;
        }

        // Build: rising energy slope with positive flux derivative.
        if self.energy_slope > 0.002 && flux_deriv > 0.0 && energy < 0.6 {
            return Section::Build;
        }

        // Sustain: high energy, stable.
        if energy > 0.6 && self.energy_slope.abs() < 0.002 {
            return Section::Sustain;
        }

        // Outro: energy dropping from a higher state toward silence.
        if self.energy_slope < -0.002 && energy < 0.25 {
            return Section::Outro;
        }

        // Stay in current section if no clear transition.
        self.current
    }

    pub fn current(&self) -> Section {
        self.current
    }
}
