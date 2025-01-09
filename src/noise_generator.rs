pub struct NoiseGenerator {
    noise_seed: u32,
}

impl NoiseGenerator {
    pub fn new() -> Self {
        Self { noise_seed: 22222 }
    }
    pub fn reset(&mut self) {
        self.noise_seed = 22222;
    }

    pub fn next_value(&mut self) -> f32 {
        self.noise_seed = self.noise_seed * 196314165 + 907633515;
        let temp = ((self.noise_seed >> 7) as i32) - 16777216;
        temp as f32 / 16777216.0f32
    }
}
