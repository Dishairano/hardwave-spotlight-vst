//! DAW-exposed parameters for Hardwave Spotlight.

use nih_plug::prelude::*;

/// Scene override — Auto lets the AI decide, others force a section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum Scene {
    #[name = "Auto"]
    Auto,
    #[name = "Intro"]
    Intro,
    #[name = "Build"]
    Build,
    #[name = "Drop"]
    Drop,
    #[name = "Breakdown"]
    Breakdown,
    #[name = "Outro"]
    Outro,
}

/// Camera angle presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum Camera {
    #[name = "Front"]
    Front,
    #[name = "Side"]
    Side,
    #[name = "Overhead"]
    Overhead,
    #[name = "Crowd"]
    Crowd,
    #[name = "Cinematic"]
    Cinematic,
    #[name = "Free"]
    Free,
}

/// Venue presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum Venue {
    #[name = "Festival"]
    Festival,
    #[name = "Club"]
    Club,
    #[name = "Arena"]
    Arena,
    #[name = "Abstract"]
    Abstract,
}

#[derive(Params)]
pub struct SpotlightParams {
    /// Overall light intensity
    #[id = "energy"]
    pub energy: FloatParam,

    /// Strobe speed (0 = off)
    #[id = "strobe_rate"]
    pub strobe_rate: FloatParam,

    /// Global color hue
    #[id = "color_hue"]
    pub color_hue: FloatParam,

    /// Color saturation (0 = white, 1 = full color)
    #[id = "color_sat"]
    pub color_sat: FloatParam,

    /// Laser brightness
    #[id = "laser"]
    pub laser: FloatParam,

    /// Moving head sweep rate
    #[id = "sweep"]
    pub sweep: FloatParam,

    /// Haze / fog level
    #[id = "fog"]
    pub fog: FloatParam,

    /// LED wall pattern cycle speed
    #[id = "led_wall"]
    pub led_wall: FloatParam,

    /// Force a specific section or let AI decide
    #[id = "scene"]
    pub scene: EnumParam<Scene>,

    /// Camera angle
    #[id = "camera"]
    pub camera: EnumParam<Camera>,

    /// Venue preset
    #[id = "venue"]
    pub venue: EnumParam<Venue>,

    /// AI drives lights (when off, knobs control directly)
    #[id = "ai_enabled"]
    pub ai_enabled: BoolParam,
}

impl Default for SpotlightParams {
    fn default() -> Self {
        Self {
            energy: FloatParam::new(
                "Energy",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            strobe_rate: FloatParam::new(
                "Strobe",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            color_hue: FloatParam::new(
                "Hue",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            color_sat: FloatParam::new(
                "Saturation",
                0.8,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            laser: FloatParam::new(
                "Laser",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            sweep: FloatParam::new(
                "Sweep",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            fog: FloatParam::new(
                "Fog",
                0.3,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            led_wall: FloatParam::new(
                "LED Wall",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            scene: EnumParam::new("Scene", Scene::Auto),
            camera: EnumParam::new("Camera", Camera::Front),
            venue: EnumParam::new("Venue", Venue::Festival),
            ai_enabled: BoolParam::new("AI", true),
        }
    }
}
