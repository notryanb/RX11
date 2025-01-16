use crate::noise_generator::NoiseGenerator;
use crate::voice::Voice;

pub const MAX_VOICES: usize = 8;

pub struct Synth {
    pub noise_mix: f32,
    pub sample_rate: f32,
    pub env_attack: f32,
    pub env_decay: f32,
    pub env_sustain: f32,
    pub env_release: f32,
    pub osc_mix: f32,
    pub detune: f32,
    pub tune: f32,
    pub pitch_bend: f32,
    pub num_voices: usize,
    noise_gen: NoiseGenerator,
    pub voices: [Voice; MAX_VOICES],
}

impl Synth {
    pub fn new() -> Self {
        Self {
            noise_mix: 0.0,
            env_attack: 0.0,
            env_decay: 0.0,
            env_sustain: 0.0,
            env_release: 0.0,
            osc_mix: 0.0,
            detune: 0.0,
            tune: 0.0,
            pitch_bend: 1.0,
            num_voices: 1,
            sample_rate: 44100.0, // TODO - Set Sample Rate from DAW
            noise_gen: NoiseGenerator::new(),
            voices: Default::default(),
        }
    }

    pub fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
        self.noise_gen.reset();
        self.pitch_bend = 1.0;
    }

    pub fn note_on(&mut self, note: i32, velocity: f32) {
        self.start_voice(0, note, velocity);
    }

    pub fn start_voice(&mut self, voice_idx: usize, note: i32, velocity: f32) {
        let period = self.calculate_period(note);

        let voice = &mut self.voices[voice_idx];
        voice.note = note;
        voice.period = period;
        voice.update_panning();

        let temp_vol = 0.5;
        voice.oscillator_1.amplitude = velocity * temp_vol;
        voice.oscillator_1.reset();
        
        voice.oscillator_2.amplitude = voice.oscillator_1.amplitude * self.osc_mix;
        voice.oscillator_2.reset();

        let env = &mut voice.envelope;
        env.attack_multiplier = self.env_attack;
        env.decay_multiplier = self.env_decay;
        env.sustain_level = self.env_sustain;
        env.release_multiplier = self.env_release;
        env.attack();
        
    }

    pub fn note_off(&mut self, note: i32) {
        let voice = &mut self.voices[0];
        if voice.note == note {
            voice.release();
        }
    }

    pub fn calculate_period(&self, note: i32) -> f32 {
        let mut period = self.tune * (-0.05776226505 * note as f32).exp();

        // Ensure the period for the detuned oscillator is at least six samples long
        while period < 6.0 || period * self.detune < 6.0 { 
            period += period; 
        }
        period
    }

    pub fn render(
        &mut self,
        output_buffer: &mut [&mut [f32]],
        block_start: usize,
        block_end: usize,
    ) {
        //self.voice.oscillator_1.period = self.voice.period * self.pitch_bend;
        //self.voice.oscillator_2.period = self.voice.oscillator_1.period * self.detune;
        for voice in &mut self.voices {
            if voice.envelope.is_active() {
                voice.oscillator_1.period = voice.period * self.pitch_bend;
                voice.oscillator_2.period = voice.oscillator_1.period * self.detune;
            }
        }

        for (_value_idx, sample_idx) in (block_start..block_end).enumerate() {
            let noise = self.noise_gen.next_value() * self.noise_mix;
            let mut output_left = 0.0;
            let mut output_right = 0.0;

            for voice in &mut self.voices {
                if voice.envelope.is_active() {
                    let output_sample = voice.render(noise);
                    output_left += output_sample * voice.pan_left;
                    output_right += output_sample * voice.pan_right;
                }
            }

            /*
            if self.voice.envelope.is_active() {
                let output_sample = self.voice.render(noise);
                output_left = output_sample * self.voice.pan_left;
                output_right = output_sample * self.voice.pan_right;
            }
            */

            output_buffer[0][sample_idx] = output_left;
            output_buffer[1][sample_idx] = output_right;
        }

        for voice in &mut self.voices {
            if !voice.envelope.is_active() {
                voice.envelope.reset();
            }
        }

        //if !self.voice.envelope.is_active() {
            //self.voice.envelope.reset();
        //}
    }
}
