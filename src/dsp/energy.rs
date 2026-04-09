//! 4-band RMS energy analyzer.
//!
//! Splits the spectrum into sub (0-80Hz), low (80-300Hz), mid (300-4kHz),
//! high (4k-20kHz) and computes smoothed RMS energy for each band.

const NUM_BANDS: usize = 4;

/// Band frequency boundaries in Hz.
const BAND_EDGES: [f32; 5] = [0.0, 80.0, 300.0, 4000.0, 20000.0];

pub struct EnergyAnalyzer {
    sample_rate: f32,
    band_energy: [f32; NUM_BANDS],
    overall_energy: f32,
    /// EMA smoothing coefficient (higher = faster response).
    alpha: f32,
}

impl EnergyAnalyzer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            band_energy: [0.0; NUM_BANDS],
            overall_energy: 0.0,
            alpha: Self::compute_alpha(sample_rate),
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
        self.alpha = Self::compute_alpha(sr);
    }

    pub fn reset(&mut self) {
        self.band_energy = [0.0; NUM_BANDS];
        self.overall_energy = 0.0;
    }

    /// Compute EMA alpha for ~50ms smoothing window.
    fn compute_alpha(sample_rate: f32) -> f32 {
        // FFT frames arrive every 2048/sr seconds.
        // We want ~50ms time constant. alpha = 1 - exp(-frame_period / tau)
        let frame_period = 2048.0 / sample_rate;
        let tau = 0.05; // 50ms
        1.0 - (-frame_period / tau).exp()
    }

    /// Process a new FFT spectrum frame (1024 dB magnitude bins).
    pub fn process_spectrum(&mut self, spectrum_db: &[f32], sample_rate: f32) {
        let bin_count = spectrum_db.len();
        let bin_hz = sample_rate / (bin_count as f32 * 2.0);

        let mut band_sums = [0.0f32; NUM_BANDS];
        let mut band_counts = [0usize; NUM_BANDS];

        for (i, &db) in spectrum_db.iter().enumerate() {
            let freq = i as f32 * bin_hz;
            // Convert dB to linear power for RMS calculation.
            let linear = if db > -120.0 {
                10.0_f32.powf(db / 20.0)
            } else {
                0.0
            };
            let power = linear * linear;

            // Find which band this bin belongs to.
            for b in 0..NUM_BANDS {
                if freq >= BAND_EDGES[b] && freq < BAND_EDGES[b + 1] {
                    band_sums[b] += power;
                    band_counts[b] += 1;
                    break;
                }
            }
        }

        // Compute RMS per band, normalize to 0..1 range.
        let mut total_energy = 0.0f32;
        for b in 0..NUM_BANDS {
            let rms = if band_counts[b] > 0 {
                (band_sums[b] / band_counts[b] as f32).sqrt()
            } else {
                0.0
            };
            // Normalize: typical max RMS from a loud signal is ~1.0.
            // Clamp to 0..1.
            let normalized = rms.min(1.0);
            self.band_energy[b] = self.band_energy[b] + self.alpha * (normalized - self.band_energy[b]);
            total_energy += self.band_energy[b];
        }

        self.overall_energy = (total_energy / NUM_BANDS as f32).min(1.0);
    }

    pub fn band_energy(&self) -> [f32; NUM_BANDS] {
        self.band_energy
    }

    pub fn overall_energy(&self) -> f32 {
        self.overall_energy
    }
}
