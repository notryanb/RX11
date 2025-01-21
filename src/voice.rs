use crate::{
    envelope::Envelope, oscillator::Oscillator, state_variable_filter::StateVariableFilter,
};

// TODO - I should probably make a constants/utils module
const PI_OVER_FOUR: f32 = std::f32::consts::PI / 4.0; //0.7853981633974483;

/// Produces the next output sample for a given note
#[derive(Default)]
pub struct Voice {
    pub note: i32,
    pub saw: f32,
    pub period: f32,
    pub pan_left: f32,
    pub pan_right: f32,
    pub target_period: f32,
    pub glide_rate: f32,
    pub cutoff_freq: f32,
    pub filter_resonance: f32,
    pub filter_mod: f32,
    pub filter_env_depth: f32,
    pub pitch_bend: f32,
    pub oscillator_1: Oscillator,
    pub oscillator_2: Oscillator,
    pub envelope: Envelope,
    pub filter_envelope: Envelope,
    pub filter: StateVariableFilter,
}

impl Voice {
    pub fn reset(&mut self) {
        self.note = 0;
        self.saw = 0.0;
        self.pan_left = 0.707;
        self.pan_right = 0.707;
        self.target_period = 0.0;
        self.oscillator_1.reset();
        self.oscillator_2.reset();
        self.envelope.reset();
        self.filter_envelope.reset();
        self.filter.reset();
    }

    // Mixes the oscillator, noise, and envelope together
    pub fn render(&mut self, input: f32) -> f32 {
        let sample_1 = self.oscillator_1.next_sample();
        let sample_2 = self.oscillator_2.next_sample();

        // This is a leaky integrator to create a sawtooth wave
        self.saw = self.saw * 0.997 + sample_1 - sample_2;

        let mut output = self.saw + input;
        output = self.filter.render(output);

        let envelope = self.envelope.next_value();
        output * envelope
        //envelope // Return only the envelope to view it in an oscilloscope
    }

    pub fn update_lfo(&mut self) {
        self.period += self.glide_rate * (self.target_period - self.period);

        let filter_env = self.filter_envelope.next_value();

        let mut modulated_cutoff = self.cutoff_freq * (self.filter_mod + self.filter_env_depth * filter_env).exp() / self.pitch_bend;
        modulated_cutoff = modulated_cutoff.clamp(30.0, 20_000.0);

        self.filter
            .update_coefficients(modulated_cutoff, self.filter_resonance);
    }

    pub fn release(&mut self) {
        self.envelope.release();
        self.filter_envelope.release();
    }

    pub fn update_panning(&mut self) {
        let panning = ((self.note as f32 - 60.0) / 24.0).clamp(-1.0, 1.0);
        self.pan_left = (PI_OVER_FOUR * (1.0 - panning)).sin();
        self.pan_right = (PI_OVER_FOUR * (1.0 + panning)).sin();
    }
}
