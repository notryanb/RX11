use crate::oscillator::Oscillator;

/// Produces the next output sample for a given note
#[derive(Default)]
pub struct Voice {
    pub note: i32,
    pub velocity: f32,
    pub saw: f32,
    pub oscillator: Oscillator,
}

impl Voice {
    pub fn reset(&mut self) {
        self.note = 0;
        self.velocity = 0.0;
        self.saw = 0.0;
        self.oscillator.reset();
    }

    pub fn render(&mut self) -> f32 {
        let mut sample = self.oscillator.next_sample();
        self.saw = self.saw * 0.997 + sample;
        self.saw
    }
}
