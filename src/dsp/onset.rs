//! Transient onset detection for kick, snare, and hihat.
//!
//! Monitors band energy for sudden spikes with adaptive thresholds and cooldowns.

pub struct OnsetDetector {
    sample_rate: f32,
    /// EMA of each band's energy (adaptive threshold baseline).
    band_avg: [f32; 4],
    /// Cooldown counters (in FFT frames) per trigger.
    cooldown: [u32; 3],
    /// Whether each onset fired this frame.
    triggers: [bool; 3],
    /// Frames since last FFT (for cooldown timing).
    frames_per_cooldown: u32,
}

/// Onset trigger indices.
const KICK: usize = 0;
const SNARE: usize = 1;
const HIHAT: usize = 2;

/// EMA alpha for adaptive threshold.
const THRESHOLD_ALPHA: f32 = 0.1;

/// Spike must exceed average by this multiplier to trigger.
const SPIKE_MULTIPLIER: f32 = 1.8;

impl OnsetDetector {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            band_avg: [0.0; 4],
            cooldown: [0; 3],
            triggers: [false; 3],
            frames_per_cooldown: Self::cooldown_frames(sample_rate),
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
        self.frames_per_cooldown = Self::cooldown_frames(sr);
    }

    pub fn reset(&mut self) {
        self.band_avg = [0.0; 4];
        self.cooldown = [0; 3];
        self.triggers = [false; 3];
    }

    /// ~50ms cooldown in FFT frames.
    fn cooldown_frames(sample_rate: f32) -> u32 {
        // Each FFT frame = 2048 samples. 50ms = 0.05 * sr samples.
        let frames = (0.05 * sample_rate / 2048.0).ceil() as u32;
        frames.max(1)
    }

    /// Process band energy [sub, low, mid, high] (each 0..1).
    pub fn process(&mut self, band_energy: &[f32; 4]) {
        // Reset triggers.
        self.triggers = [false; 3];

        // Decrement cooldowns.
        for cd in self.cooldown.iter_mut() {
            *cd = cd.saturating_sub(1);
        }

        // Update adaptive thresholds and check for spikes.
        for (i, &energy) in band_energy.iter().enumerate() {
            let old_avg = self.band_avg[i];
            self.band_avg[i] = old_avg + THRESHOLD_ALPHA * (energy - old_avg);
        }

        // Kick: sub band (0) spike.
        if self.cooldown[KICK] == 0 && band_energy[0] > self.band_avg[0] * SPIKE_MULTIPLIER && band_energy[0] > 0.05 {
            self.triggers[KICK] = true;
            self.cooldown[KICK] = self.frames_per_cooldown;
        }

        // Snare: low-mid band (1) spike, often with mid (2) content.
        if self.cooldown[SNARE] == 0 && band_energy[1] > self.band_avg[1] * SPIKE_MULTIPLIER && band_energy[1] > 0.04 {
            self.triggers[SNARE] = true;
            self.cooldown[SNARE] = self.frames_per_cooldown;
        }

        // Hihat: high band (3) spike.
        if self.cooldown[HIHAT] == 0 && band_energy[3] > self.band_avg[3] * SPIKE_MULTIPLIER && band_energy[3] > 0.03 {
            self.triggers[HIHAT] = true;
            self.cooldown[HIHAT] = self.frames_per_cooldown;
        }
    }

    pub fn kick(&self) -> bool { self.triggers[KICK] }
    pub fn snare(&self) -> bool { self.triggers[SNARE] }
    pub fn hihat(&self) -> bool { self.triggers[HIHAT] }
}
