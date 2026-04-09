//! Beat tracking via autocorrelation of onset energy.
//!
//! Maintains a ~4-second history of energy values, performs autocorrelation
//! to find the dominant periodicity in the 60-200 BPM range, and tracks
//! beat phase by counting samples between beats.

const HISTORY_SIZE: usize = 256; // ~4s at 2048-sample FFT frames / 44.1kHz

pub struct BeatTracker {
    sample_rate: f32,
    /// Circular buffer of overall energy values (one per FFT frame).
    history: Vec<f32>,
    write_pos: usize,
    /// Detected BPM.
    bpm: f32,
    /// Beat phase 0..1 (0 = on beat, 0.5 = off-beat).
    phase: f32,
    /// Confidence 0..1 of the BPM estimate.
    confidence: f32,
    /// Samples since last detected beat (for phase tracking).
    samples_since_beat: f32,
    /// Expected samples per beat at current BPM.
    samples_per_beat: f32,
    /// Frame counter for throttled autocorrelation.
    frame_counter: u32,
}

impl BeatTracker {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            history: vec![0.0; HISTORY_SIZE],
            write_pos: 0,
            bpm: 0.0,
            phase: 0.0,
            confidence: 0.0,
            samples_since_beat: 0.0,
            samples_per_beat: 0.0,
            frame_counter: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
        self.reset();
    }

    pub fn reset(&mut self) {
        self.history.iter_mut().for_each(|s| *s = 0.0);
        self.write_pos = 0;
        self.bpm = 0.0;
        self.phase = 0.0;
        self.confidence = 0.0;
        self.samples_since_beat = 0.0;
        self.samples_per_beat = 0.0;
        self.frame_counter = 0;
    }

    /// Called once per FFT frame with the overall energy (0..1).
    pub fn process(&mut self, energy: f32) {
        // Store energy in circular buffer.
        self.history[self.write_pos] = energy;
        self.write_pos = (self.write_pos + 1) % HISTORY_SIZE;

        // Update phase tracking.
        let frame_samples = 2048.0; // FFT hop size
        self.samples_since_beat += frame_samples;

        if self.samples_per_beat > 0.0 {
            self.phase = (self.samples_since_beat / self.samples_per_beat).fract();
            // Detect beat onset (phase wraps around).
            if self.samples_since_beat >= self.samples_per_beat {
                self.samples_since_beat -= self.samples_per_beat;
            }
        }

        // Run autocorrelation every 8 frames (~370ms) to save CPU.
        self.frame_counter += 1;
        if self.frame_counter >= 8 {
            self.frame_counter = 0;
            self.compute_autocorrelation();
        }
    }

    fn compute_autocorrelation(&mut self) {
        let frames_per_second = self.sample_rate / 2048.0;

        // BPM range 60-200 corresponds to lag range in frames.
        let min_lag = (frames_per_second * 60.0 / 200.0) as usize; // ~200 BPM
        let max_lag = (frames_per_second * 60.0 / 60.0) as usize;  // ~60 BPM
        let max_lag = max_lag.min(HISTORY_SIZE / 2);

        if min_lag >= max_lag { return; }

        // Compute mean of history.
        let mean: f32 = self.history.iter().sum::<f32>() / HISTORY_SIZE as f32;

        // Compute autocorrelation for each lag.
        let mut best_lag = min_lag;
        let mut best_corr = f32::NEG_INFINITY;
        let mut zero_lag_corr = 0.0f32;

        for lag in 0..=max_lag {
            let mut sum = 0.0f32;
            let n = HISTORY_SIZE - lag;
            for i in 0..n {
                let a = self.history[(self.write_pos + i) % HISTORY_SIZE] - mean;
                let b = self.history[(self.write_pos + i + lag) % HISTORY_SIZE] - mean;
                sum += a * b;
            }
            let corr = sum / n as f32;

            if lag == 0 {
                zero_lag_corr = corr;
            }

            if lag >= min_lag && corr > best_corr {
                best_corr = corr;
                best_lag = lag;
            }
        }

        // Convert lag to BPM.
        if best_lag > 0 {
            let beat_period_sec = best_lag as f32 / frames_per_second;
            let new_bpm = 60.0 / beat_period_sec;

            // Confidence is the normalized correlation.
            let conf = if zero_lag_corr > 1e-6 {
                (best_corr / zero_lag_corr).max(0.0).min(1.0)
            } else {
                0.0
            };

            // Smooth BPM updates to avoid jitter.
            if self.bpm == 0.0 || conf > 0.3 {
                self.bpm = self.bpm * 0.7 + new_bpm * 0.3;
                self.confidence = self.confidence * 0.8 + conf * 0.2;
                self.samples_per_beat = 60.0 / self.bpm * self.sample_rate;
            }
        }
    }

    pub fn bpm(&self) -> f32 { self.bpm }
    pub fn phase(&self) -> f32 { self.phase }
    pub fn confidence(&self) -> f32 { self.confidence }
}
