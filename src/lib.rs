//! Hardwave Spotlight — AI-driven concert lightshow generator VST3/CLAP plugin.
//!
//! Audio passes through untouched. DSP analyses the signal for energy, onsets,
//! beat phase, spectral flux, and section classification. Analysis data is sent
//! to the webview at ~60 fps where a Three.js scene renders a reactive lightshow.

use crossbeam_channel::{Sender, Receiver};
use nih_plug::prelude::*;
use parking_lot::Mutex;
use std::num::NonZeroU32;
use std::sync::Arc;

mod auth;
mod dsp;
mod editor;
mod params;
mod protocol;

use dsp::{
    BeatTracker, EnergyAnalyzer, OnsetDetector, SectionClassifier, SpectrumAnalyzer, SpectralFlux,
};
use params::SpotlightParams;
use protocol::SpotlightPacket;

struct HardwaveSpotlight {
    params: Arc<SpotlightParams>,

    // DSP analysis modules.
    analyzer: SpectrumAnalyzer,
    energy: EnergyAnalyzer,
    flux: SpectralFlux,
    onset: OnsetDetector,
    beat: BeatTracker,
    section: SectionClassifier,

    // Editor communication.
    editor_packet_tx: Sender<SpotlightPacket>,
    editor_packet_rx: Arc<Mutex<Receiver<SpotlightPacket>>>,
    update_counter: u32,

    // Spectrum send throttle (send every Nth packet to save bandwidth).
    spectrum_counter: u32,

    sample_rate: f32,
}

impl Default for HardwaveSpotlight {
    fn default() -> Self {
        let sr = 44100.0;
        let (pkt_tx, pkt_rx) = crossbeam_channel::bounded(4);
        Self {
            params: Arc::new(SpotlightParams::default()),
            analyzer: SpectrumAnalyzer::new(sr),
            energy: EnergyAnalyzer::new(sr),
            flux: SpectralFlux::new(),
            onset: OnsetDetector::new(sr),
            beat: BeatTracker::new(sr),
            section: SectionClassifier::new(sr),
            editor_packet_tx: pkt_tx,
            editor_packet_rx: Arc::new(Mutex::new(pkt_rx)),
            update_counter: 0,
            spectrum_counter: 0,
            sample_rate: sr,
        }
    }
}

impl Plugin for HardwaveSpotlight {
    const NAME: &'static str = "Hardwave Spotlight";
    const VENDOR: &'static str = "Hardwave Studios";
    const URL: &'static str = "https://hardwavestudios.com";
    const EMAIL: &'static str = "hello@hardwavestudios.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let token = auth::load_token();
        Some(Box::new(editor::SpotlightEditor::new(
            Arc::clone(&self.params),
            Arc::clone(&self.editor_packet_rx),
            token,
        )))
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let sr = buffer_config.sample_rate;
        self.sample_rate = sr;
        self.analyzer.set_sample_rate(sr);
        self.energy.set_sample_rate(sr);
        self.onset.set_sample_rate(sr);
        self.beat.set_sample_rate(sr);
        self.section.set_sample_rate(sr);
        true
    }

    fn reset(&mut self) {
        self.analyzer.reset();
        self.energy.reset();
        self.flux.reset();
        self.onset.reset();
        self.beat.reset();
        self.section.reset();
        self.update_counter = 0;
        self.spectrum_counter = 0;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Snapshot params for the packet.
        let pkt_snapshot = editor::snapshot_params(&self.params);

        for mut frame in buffer.iter_samples() {
            let num_channels = frame.len();
            if num_channels < 2 { continue; }

            // Read audio but DO NOT modify — pass-through.
            let l = *frame.get_mut(0).unwrap();
            let r = *frame.get_mut(1).unwrap();
            let mono = (l + r) * 0.5;

            // Feed analyzer.
            self.analyzer.push_sample(mono);

            // When a new FFT frame is ready, run analysis chain.
            if let Some(spectrum) = self.analyzer.get_spectrum() {
                self.energy.process_spectrum(&spectrum, self.analyzer.sample_rate());
                self.flux.process(&spectrum);
                self.onset.process(&self.energy.band_energy());
                self.beat.process(self.energy.overall_energy());
                self.section.process(
                    self.energy.overall_energy(),
                    self.flux.flux(),
                    self.flux.flux_derivative(),
                    self.beat.bpm(),
                );
            }
        }

        // Send state packet to editor (~60 fps).
        self.update_counter += 1;
        if self.update_counter >= 4 {
            self.update_counter = 0;

            let mut packet = pkt_snapshot;

            // Fill analysis data.
            packet.band_energy = self.energy.band_energy();
            packet.overall_energy = self.energy.overall_energy();
            packet.kick = self.onset.kick();
            packet.snare = self.onset.snare();
            packet.hihat = self.onset.hihat();
            packet.bpm = self.beat.bpm();
            packet.beat_phase = self.beat.phase();
            packet.beat_confidence = self.beat.confidence();
            packet.spectral_flux = self.flux.flux();
            packet.section = self.section.current().to_string();

            // Send spectrum every 4th packet (~15 fps) to save bandwidth.
            self.spectrum_counter += 1;
            if self.spectrum_counter >= 4 {
                self.spectrum_counter = 0;
                packet.spectrum = self.analyzer.get_spectrum();
            }

            let _ = self.editor_packet_tx.try_send(packet);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for HardwaveSpotlight {
    const CLAP_ID: &'static str = "com.hardwavestudios.spotlight";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("AI-driven concert lightshow generator");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = Some("https://hardwavestudios.com/support");
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Analyzer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for HardwaveSpotlight {
    const VST3_CLASS_ID: [u8; 16] = *b"HWSpotlight_v001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Analyzer,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(HardwaveSpotlight);
nih_export_vst3!(HardwaveSpotlight);
