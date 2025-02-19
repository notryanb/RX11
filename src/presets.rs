use crate::{GlideMode, PolyMode};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Preset {
    pub name: String,
    pub values: HashMap<String, f32>,
}

impl Preset {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            values: HashMap::new(),
        }
    }

    pub fn add_param(mut self, param_name: &str, param_val: f32) -> Self {
        self.values.insert(param_name.to_string(), param_val);
        self
    }
}

#[derive(Clone)]
pub struct Presets(pub Vec<Preset>);

impl Presets {
    pub fn init() -> Self {
        Self(vec![
            Preset::new("Init")
                .add_param("osc_mix", 0.0)
                .add_param("osc_tune", -12.0)
                .add_param("osc_fine_tune", 0.0)
                .add_param("glide_mode", GlideMode::to_f32(GlideMode::Off))
                .add_param("glide_rate", 35.0)
                .add_param("glide_bend", 0.0)
                .add_param("filter_freq", 100.0)
                .add_param("filter_reso", 15.0)
                .add_param("filter_env", 50.0)
                .add_param("filter_lfo", 0.0)
                .add_param("filter_velocity", 0.0)
                .add_param("filter_attack", 0.0)
                .add_param("filter_decay", 30.0)
                .add_param("filter_sustain", 0.0)
                .add_param("filter_release", 25.0)
                .add_param("env_attack", 0.0)
                .add_param("env_decay", 50.0)
                .add_param("env_sustain", 100.0)
                .add_param("env_release", 30.0)
                .add_param("lfo_rate", 0.81)
                .add_param("vibrato", 0.0)
                .add_param("noise", 0.0)
                .add_param("octave", 0.0)
                .add_param("tuning", 0.0)
                .add_param("output", 1.0)
                .add_param("poly_mode", PolyMode::to_f32(PolyMode::Poly)),
            Preset::new("5th Sweep Pad")
                .add_param("osc_mix", 100.0)
                .add_param("osc_tune", -7.0)
                .add_param("osc_fine_tune", -6.30)
                .add_param("glide_mode", GlideMode::to_f32(GlideMode::Legato))
                .add_param("glide_rate", 32.0)
                .add_param("glide_bend", 0.0)
                .add_param("filter_freq", 90.0)
                .add_param("filter_reso", 60.0)
                .add_param("filter_env", -76.0)
                .add_param("filter_lfo", 0.0)
                .add_param("filter_velocity", 0.0)
                .add_param("filter_attack", 90.0)
                .add_param("filter_decay", 89.0)
                .add_param("filter_sustain", 90.0)
                .add_param("filter_release", 73.0)
                .add_param("env_attack", 0.0)
                .add_param("env_decay", 50.0)
                .add_param("env_sustain", 100.0)
                .add_param("env_release", 71.0)
                .add_param("lfo_rate", 0.81)
                .add_param("vibrato", 30.0)
                .add_param("noise", 0.0)
                .add_param("octave", 0.0)
                .add_param("tuning", 0.0)
                .add_param("output", 1.0)
                .add_param("poly_mode", PolyMode::to_f32(PolyMode::Poly)),
            Preset::new("Echo Pad [SA]")
                .add_param("osc_mix", 88.0)
                .add_param("osc_tune", 0.0)
                .add_param("osc_fine_tune", 0.0)
                .add_param("glide_mode", GlideMode::to_f32(GlideMode::Off))
                .add_param("glide_rate", 49.0)
                .add_param("glide_bend", 0.0)
                .add_param("filter_freq", 46.0)
                .add_param("filter_reso", 76.0)
                .add_param("filter_env", 38.0)
                .add_param("filter_lfo", 10.0)
                .add_param("filter_velocity", 38.0)
                .add_param("filter_attack", 100.0)
                .add_param("filter_decay", 86.0)
                .add_param("filter_sustain", 76.0)
                .add_param("filter_release", 57.0)
                .add_param("env_attack", 30.0)
                .add_param("env_decay", 80.0)
                .add_param("env_sustain", 68.0)
                .add_param("env_release", 66.0)
                .add_param("lfo_rate", 0.79)
                .add_param("vibrato", -74.0)
                .add_param("noise", 25.0)
                .add_param("octave", 0.0)
                .add_param("tuning", 0.0)
                .add_param("output", 1.0)
                .add_param("poly_mode", PolyMode::to_f32(PolyMode::Poly)),
        ])
    }
}
