use crate::envelope::Envelope;
use crate::oscillator::Oscillator;

/// Produces the next output sample for a given note
#[derive(Default)]
pub struct Voice {
    pub note: i32,
    pub saw: f32,
    pub oscillator_1: Oscillator,
    pub oscillator_2: Oscillator,
    pub envelope: Envelope,
}

impl Voice {
    pub fn reset(&mut self) {
        self.note = 0;
        self.saw = 0.0;
        self.oscillator_1.reset();
        self.oscillator_2.reset();
        self.envelope.reset();
    }

    // Mixes the oscillator, noise, and envelope together
    pub fn render(&mut self, input: f32) -> f32 {
        let sample_1 = self.oscillator_1.next_sample();
        let sample_2 = self.oscillator_2.next_sample();

        // This is a leaky integrator
        self.saw = self.saw * 0.997 + sample_1 - sample_2;

        let output = self.saw + input;

        let envelope = self.envelope.next_value();
        output * envelope
        //envelope // Return only the envelope to view it in an oscilloscope
    }

    pub fn release(&mut self) {
        self.envelope.release();
    }
}
