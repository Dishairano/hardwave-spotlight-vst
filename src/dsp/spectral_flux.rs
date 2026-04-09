//! Spectral flux — measures frame-to-frame spectral change.
//!
//! Half-wave rectified sum of positive spectral differences with EMA smoothing.

pub struct SpectralFlux {
    prev_spectrum: Vec<f32>,
    flux: f32,
    flux_prev: f32,
    flux_derivative: f32,
    alpha: f32,
}

impl SpectralFlux {
    pub fn new() -> Self {
        Self {
            prev_spectrum: Vec::new(),
            flux: 0.0,
            flux_prev: 0.0,
            flux_derivative: 0.0,
            alpha: 0.15,
        }
    }

    pub fn reset(&mut self) {
        self.prev_spectrum.clear();
        self.flux = 0.0;
        self.flux_prev = 0.0;
        self.flux_derivative = 0.0;
    }

    /// Process a new spectrum frame. Input: dB magnitude bins.
    pub fn process(&mut self, spectrum_db: &[f32]) {
        if self.prev_spectrum.len() != spectrum_db.len() {
            self.prev_spectrum = spectrum_db.to_vec();
            return;
        }

        // Half-wave rectified flux: only positive changes (onset-like).
        let mut raw_flux = 0.0f32;
        for (i, &cur) in spectrum_db.iter().enumerate() {
            let diff = cur - self.prev_spectrum[i];
            if diff > 0.0 {
                raw_flux += diff;
            }
        }

        // Normalize by bin count.
        raw_flux /= spectrum_db.len() as f32;

        // EMA smoothing.
        self.flux_prev = self.flux;
        self.flux = self.flux + self.alpha * (raw_flux - self.flux);
        self.flux_derivative = self.flux - self.flux_prev;

        self.prev_spectrum.copy_from_slice(spectrum_db);
    }

    pub fn flux(&self) -> f32 {
        self.flux
    }

    pub fn flux_derivative(&self) -> f32 {
        self.flux_derivative
    }
}
