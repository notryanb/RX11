use crate::noise_generator::NoiseGenerator;
use crate::voice::Voice;

pub struct Synth {
    pub noise_mix: f32,
    pub sample_rate: f32,
    noise_gen: NoiseGenerator,
    pub voice: Voice,
}

impl Synth {
    pub fn new() -> Self {
        Self {
            noise_mix: 0.0,
            sample_rate: 44100.0, // TODO - Set Sample Rate from DAW
            noise_gen: NoiseGenerator::new(),
            voice: Default::default(),
        }
    }

    pub fn reset(&mut self) {
        self.voice.reset();
        self.noise_gen.reset();
    }

    pub fn note_on(&mut self, note: i32, velocity: f32) {
        self.voice.note = note;

        // Map the MIDI note: [0..128] to frequency
        // 440 * 2^((note - 69) / 12)
        let frequency = 440.0 * ((note - 69) as f32 / 12.0).exp2();

        // TODO - Expose these as methods on the voice or maybe on the synth itself?
        let temp_vol = 0.5;
        self.voice.oscillator.amplitude = velocity * temp_vol;
        self.voice.oscillator.period = self.sample_rate / frequency;
        self.voice.oscillator.reset();
    }

    pub fn note_off(&mut self, note: i32) {
        if self.voice.note == note {
            self.voice.note = 0;
            self.voice.velocity = 0.0;
        }
    }

    pub fn render(
        &mut self,
        output_buffer: &mut [&mut [f32]],
        block_start: usize,
        block_end: usize,
    ) {
        for (value_idx, sample_idx) in (block_start..block_end).enumerate() {
            let noise = self.noise_gen.next_value() * self.noise_mix;
            let mut output_sample = 0.0f32;

            if self.voice.note > 0 {
                output_sample = self.voice.render() + noise;
            }

            output_buffer[0][sample_idx] += output_sample;
            output_buffer[1][sample_idx] += output_sample;
        }
    }
}
