use crate::noise_generator::NoiseGenerator;
use crate::voice::Voice;

pub const MAX_VOICES: usize = 8;
pub const ANALOG: f32 = 0.002;
pub const SUSTAIN: i32 = -1;

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
    pub volume_trim: f32,
    pub output_level: f32,
    pub num_voices: usize,
    pub is_sustained: bool,
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
            volume_trim: 1.0,
            output_level: 0.0,
            num_voices: 1,
            is_sustained: false,
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
        let mut voice_idx: usize = 0;

        if self.num_voices > 1 {
            voice_idx = self.find_free_voice();
        }

        self.start_voice(voice_idx, note, velocity);
    }

    // Finds the quietest voice not in attack
    // TODO - I wish I could do this with Option<&mut Voice>, but I'm having mut borrow issues
    // Notes:
    // This allows the same note to be played in multiple voices if the same note is played in succession multiple times.
    // Some voice stealing ideas.
    // If same note was playing, reuse the voice
    // Try to steal released notes
    // Steal the note w/ smallest amplitude or velocity
    // Steal the oldest note which isn't either the highest or lowest note being played
    pub fn find_free_voice(&mut self) -> usize {
        let mut voice_idx: usize = 0;
        let mut loudness = 100.0; // Louder than any envelope

        for (idx, voice) in self.voices.iter().enumerate() {
            if voice.envelope.level < loudness && !voice.envelope.is_in_attack() {
                loudness = voice.envelope.level;
                voice_idx = idx;
            }
        }

        voice_idx
    }

    pub fn start_voice(&mut self, voice_idx: usize, note: i32, velocity: f32) {
        let period = self.calculate_period(voice_idx, note);

        let voice = &mut self.voices[voice_idx];
        voice.note = note;
        voice.period = period;
        voice.update_panning();

        voice.oscillator_1.amplitude = velocity * self.volume_trim;
        voice.oscillator_1.reset();
        
        voice.oscillator_2.amplitude = voice.oscillator_1.amplitude * self.osc_mix;
        voice.oscillator_2.reset();

        //let env = &mut voice.envelope;
        voice.envelope.attack_multiplier = self.env_attack;
        voice.envelope.decay_multiplier = self.env_decay;
        voice.envelope.sustain_level = self.env_sustain;
        voice.envelope.release_multiplier = self.env_release;
        voice.envelope.attack();
        
    }

    pub fn note_off(&mut self, note: i32) {
        for voice in &mut self.voices {
            if voice.note == note {
                if self.is_sustained {
                    voice.note = SUSTAIN;                    
                } else {
                    voice.release();
                    voice.note = 0;
                }
            }
        }
    }

    pub fn calculate_period(&self, voice_idx: usize, note: i32) -> f32 {
        // Adding the ANALOG "randomness" will slightly detune the note to make it sound more analog
        let mut period = self.tune * (-0.05776226505 * (note as f32 + ANALOG * voice_idx as f32)).exp();

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

            output_left *= self.output_level;
            output_right *= self.output_level;

            output_buffer[0][sample_idx] = output_left;
            output_buffer[1][sample_idx] = output_right;
        }

        for voice in &mut self.voices {
            if !voice.envelope.is_active() {
                voice.envelope.reset();
            }
        }
    }
}
