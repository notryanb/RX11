#[derive(Default)]
pub struct Envelope {
    pub level: f32,
}

impl Envelope {
    pub fn next_value(&mut self) -> f32 {
        self.level *= 0.9999;
        self.level
    }
}
