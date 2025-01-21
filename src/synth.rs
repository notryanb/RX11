use crate::noise_generator::NoiseGenerator;
use crate::voice::Voice;
use crate::RX11Params;

pub const MAX_VOICES: usize = 8;
pub const ANALOG: f32 = 0.002;
pub const SUSTAIN: i32 = -1;
pub const LFO_MAX: f32 = 32.0;

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
    pub velocity_sensitivity: f32,
    pub vibrato: f32,
    pub pwm_depth: f32,
    pub lfo_phase_increment: f32,
    pub lfo: f32,
    pub lfo_step: i32,
    pub mod_wheel: f32,
    pub glide_mode: crate::GlideMode,
    pub glide_rate: f32,
    pub glide_bend: f32,
    pub last_note: i32,
    pub filter_key_tracking: f32,
    pub filter_resonance: f32,
    pub filter_lfo_depth: f32,
    pub filter_ctrl: f32,
    pub filter_smoothing: f32,
    pub filter_attack: f32,
    pub filter_decay: f32,
    pub filter_sustain: f32,
    pub filter_release: f32,
    pub filter_env_depth: f32,
    pub pressure: f32,
    pub num_voices: usize,
    pub is_sustained: bool,
    pub ignore_velocity: bool,
    noise_gen: NoiseGenerator,
    pub voices: [Voice; MAX_VOICES],
}

impl Synth {
    pub fn new() -> Self {
        Self {
            sample_rate: 44100.0, // This is called from the Plugin Default... so have to figure out how to pass in the context transport?
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
            vibrato: 0.0,
            pwm_depth: 0.0,
            last_note: 0,
            lfo: 0.0,
            lfo_step: 0,
            lfo_phase_increment: 0.0,
            velocity_sensitivity: 0.0,
            mod_wheel: 0.0,
            glide_mode: crate::GlideMode::Off,
            glide_rate: 1.0,
            glide_bend: 0.0,
            filter_key_tracking: 0.0,
            filter_resonance: 0.0,
            filter_lfo_depth: 0.0,
            filter_ctrl: 0.0,
            filter_smoothing: 0.0,
            filter_attack: 0.0,
            filter_decay: 0.0,
            filter_sustain: 0.0,
            filter_release: 0.0,
            filter_env_depth: 0.0,
            pressure: 0.0,
            num_voices: 1,
            is_sustained: false,
            ignore_velocity: false,
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
        self.mod_wheel = 0.0;
        self.lfo = 0.0;
        self.lfo_step = 0;
        self.last_note = 0;
        self.filter_ctrl = 0.0;
        self.pressure = 0.0;
        self.filter_smoothing = 0.0;
    }

    pub fn note_on(&mut self, note: i32, velocity: f32) {
        let mut velocity = velocity; // Shadow the variable so it can be mutateble without changing the signature

        if self.ignore_velocity {
            velocity = 80.0;
        }

        let mut voice_idx: usize = 0;

        if self.num_voices == 1 {
            // MONOPHONIC
            if self.voices[0].note > 0 {
                // Legato style
                self.shift_queued_notes();
                self.restart_mono_voice(note, velocity);
                return;
            }
        } else {
            // POLYPHONIC
            voice_idx = self.find_free_voice();
        }

        self.start_voice(voice_idx, note, velocity);
    }

    // FIXME: Add test cases
    pub fn shift_queued_notes(&mut self) {
        for tmp in (0..MAX_VOICES).rev() {
            self.voices[tmp].note = self.voices[tmp - 1].note;
        }
    }

    // FIXME: Add test cases
    pub fn next_queued_note(&mut self) -> i32 {
        let mut held: usize = 0;

        for i in (0..MAX_VOICES).rev() {
            if self.voices[i].note > 0 {
                held = i;
            }
        }

        if held > 0 {
            let note = self.voices[held].note;
            self.voices[held].note = 0;
            return note;
        }

        0
    }

    // Finds the quietest voice not in attack
    // TODO - I wish I could do this with Option<&mut Voice>, but I'm having mut borrow issues
    // From Rust playground, I think this can be achieved if the method is not implemented on the synth. Maybe a Voices
    // type is worth it?
    //
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

    // TODO - I'm subtracting 64 from velocity, which I believe is already 0..1. Discover how
    // this parameter might have to be changed
    pub fn start_voice(&mut self, voice_idx: usize, note: i32, velocity: f32) {
        let period = self.calculate_period(voice_idx, note);
        let is_playing_legato_style = self.is_playing_legato_style();

        let voice = &mut self.voices[voice_idx];
        voice.target_period = period;

        // dividing by PI was part of the original JX11 and makes the cutoff about 3x lower than the played note
        voice.cutoff_freq = self.sample_rate / (period * std::f32::consts::PI);
        if velocity > 0.0 {
            voice.cutoff_freq *= (self.velocity_sensitivity * (velocity - 64.0)).exp();
        }

        // Glide
        let mut note_distance = 0;
        if self.last_note > 0 {
            if self.glide_mode == crate::GlideMode::Legato
                || (self.glide_mode == crate::GlideMode::Always && is_playing_legato_style)
            {
                note_distance = note - self.last_note;
            }
        }

        voice.period = period * 1.059463094359_f32.powf(note_distance as f32 - self.glide_bend);

        if voice.period < 6.0 {
            voice.period = 6.0;
        }

        self.last_note = note;
        voice.note = note;
        voice.update_panning();

        // Adjust velocity to be non-linear - somewhat parabolic
        let velocity = 0.004 * (velocity + 64.0) * (velocity + 64.0) - 8.0;
        voice.oscillator_1.amplitude = velocity * self.volume_trim;
        voice.oscillator_1.reset();

        voice.oscillator_2.amplitude = voice.oscillator_1.amplitude * self.osc_mix;
        voice.oscillator_2.reset();

        if self.vibrato == 0.0 && self.pwm_depth > 0.0 {
            voice
                .oscillator_2
                .square_wave(&voice.oscillator_1, voice.period);
        }

        voice.envelope.attack_multiplier = self.env_attack;
        voice.envelope.decay_multiplier = self.env_decay;
        voice.envelope.sustain_level = self.env_sustain;
        voice.envelope.release_multiplier = self.env_release;
        voice.envelope.attack();

        voice.filter_envelope.attack_multiplier = self.filter_attack;
        voice.filter_envelope.decay_multiplier = self.filter_decay;
        voice.filter_envelope.sustain_level = self.filter_sustain;
        voice.filter_envelope.release_multiplier = self.filter_release;
        voice.filter_envelope.attack();
    }

    pub fn note_off(&mut self, note: i32) {
        if self.num_voices == 1 && self.voices[0].note == note {
            let queued_note = self.next_queued_note();

            if queued_note > 0 {
                self.restart_mono_voice(queued_note, -1.0);
            }
        }
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
        let mut period =
            self.tune * (-0.05776226505 * (note as f32 + ANALOG * voice_idx as f32)).exp();

        // Ensure the period for the detuned oscillator is at least six samples long
        while period < 6.0 || period * self.detune < 6.0 {
            period += period;
        }
        period
    }

    pub fn restart_mono_voice(&mut self, note: i32, velocity: f32) {
        let period = self.calculate_period(0, note);

        let voice = &mut self.voices[0];
        voice.target_period = period;

        if self.glide_mode == crate::GlideMode::Off {
            voice.period = period;
        }

        voice.envelope.level += crate::envelope::SILENCE + crate::envelope::SILENCE;
        voice.note = note;
        voice.update_panning();
    }

    pub fn update_lfo(&mut self) {
        self.lfo_step -= 1;
        if self.lfo_step <= 0 {
            self.lfo_step = LFO_MAX as i32;

            self.lfo += self.lfo_phase_increment;

            if self.lfo > std::f32::consts::PI {
                self.lfo -= std::f32::consts::TAU;
            }

            let sine = self.lfo.sin();
            let vibrato_mod = 1.0 + sine * (self.mod_wheel + self.vibrato);
            let pwm = 1.0 + sine * (self.mod_wheel + self.pwm_depth);
            let filter_mod = self.filter_key_tracking + self.filter_ctrl + (self.filter_lfo_depth + self.pressure) * sine;
            self.filter_smoothing += 0.005 * (filter_mod - self.filter_smoothing);

            for voice in &mut self.voices {
                if voice.envelope.is_active() {
                    voice.oscillator_1.modulation = vibrato_mod;
                    voice.oscillator_2.modulation = pwm;
                    voice.filter_mod = self.filter_smoothing;
                    voice.update_lfo();

                    //self.update_period(voice); // TODO: This causes mut borrow issues
                    voice.oscillator_1.period = voice.period * self.pitch_bend;
                    voice.oscillator_2.period = voice.oscillator_1.period * self.detune;
                }
            }
        }
    }

    pub fn is_playing_legato_style(&self) -> bool {
        let mut held = 0;

        for i in 0..MAX_VOICES {
            if self.voices[i].note > 0 {
                held += 1;
            }
        }

        return held > 0;
    }

    // pub fn update_period(&mut self, voice: &mut Voice) {
    //     voice.oscillator_1.period = voice.period * self.pitch_bend;
    //     voice.oscillator_2.period = voice.oscillator_1.period * self.detune;
    // }

    pub fn render(
        &mut self,
        output_buffer: &mut [&mut [f32]],
        block_start: usize,
        block_end: usize,
        params: &RX11Params,
    ) {
        for voice in &mut self.voices {
            if voice.envelope.is_active() {
                voice.oscillator_1.period = voice.period * self.pitch_bend;
                voice.oscillator_2.period = voice.oscillator_1.period * self.detune;
                voice.glide_rate = self.glide_rate;
                voice.filter_resonance = self.filter_resonance;
                voice.pitch_bend = self.pitch_bend;
                voice.filter_env_depth = self.filter_env_depth;
            }
        }

        for (_value_idx, sample_idx) in (block_start..block_end).enumerate() {
            self.update_lfo();

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

            // TODO: See if there is ever a case where the buffer is "MONO" where
            // there is no right channel. The sample needs to be (output_left + output_right) * 0.5
            let output_level = params.output_level.smoothed.next();
            output_left *= output_level;
            output_right *= output_level;

            output_buffer[0][sample_idx] = output_left;
            output_buffer[1][sample_idx] = output_right;
        }

        for voice in &mut self.voices {
            if !voice.envelope.is_active() {
                voice.envelope.reset();
                voice.filter.reset();
            }
        }
    }
}
