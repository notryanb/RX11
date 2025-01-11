use crate::envelope::Envelope;
use crate::oscillator::Oscillator;

/// Produces the next output sample for a given note
#[derive(Default)]
pub struct Voice {
    pub note: i32,
    pub saw: f32,
    pub oscillator: Oscillator,
    pub envelope: Envelope,
}

impl Voice {
    pub fn reset(&mut self) {
        self.note = 0;
        self.saw = 0.0;
        self.oscillator.reset();
        self.envelope.reset();
    }

    // Mixes the oscillator, noise, and envelope together
    pub fn render(&mut self, input: f32) -> f32 {
        let sample = self.oscillator.next_sample();

        // This is a leaky integrator
        self.saw = self.saw * 0.997 + sample;

        let output = self.saw + input;

        let envelope = self.envelope.next_value();
        output * envelope
        //envelope // Return only the envelope to view it in an oscilloscope
    }

    pub fn release(&mut self) {
        self.envelope.release();
    }
}
