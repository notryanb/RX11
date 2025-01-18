#[derive(Default)]
pub struct StateVariableFilter {
    pub sample_rate: f32,
    g: f32,
    k: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    ic1eq: f32,
    ic2eq: f32,
}

impl StateVariableFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            ..Default::default()
        }
    }

    pub fn reset(&mut self) {
        self.g = 0.0;
        self.k = 0.0;
        self.a1 = 0.0;
        self.a2 = 0.0;
        self.a3 = 0.0;

        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    pub fn update_coefficients(&mut self, cutoff_freq: f32, resonance: f32) {
        self.g = (std::f32::consts::PI * cutoff_freq / self.sample_rate).tan();
        self.k = 1.0 / resonance;
        self.a1 = 1.0 / (1.0 + self.g * (self.g + self.k));
        self.a2 = self.g * self.a1;
        self.a3 = self.g * self.a2;
    }

    pub fn render(&mut self, input_sample: f32) -> f32 {
        let v3 = input_sample - self.ic2eq;
        let v1 = self.a1 * self.ic1eq + self.a2 * v3;
        let v2 = self.ic2eq + self.a2 * self.ic1eq + self.a3 * v3;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        v2
    }
}
