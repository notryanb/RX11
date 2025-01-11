const SILENCE: f32 = 0.0001; // -80db = 20 * log(0.0001)

#[derive(Default)]
pub struct Envelope {
    pub level: f32,
    pub attack_multiplier: f32,
    pub decay_multiplier: f32,
    pub sustain_level: f32,
    pub release_multiplier: f32,
    target: f32,
    multiplier: f32,
}

impl Envelope {
    pub fn reset(&mut self) {
        self.level = 0.0;
        self.target = 0.0;
        self.multiplier = 0.0;
    }

    pub fn next_value(&mut self) -> f32 {
        // One-pole filter
        self.level = self.multiplier * (self.level - self.target) + self.target;

        if self.level + self.target > 3.0 {
            self.multiplier = self.decay_multiplier;
            self.target = self.sustain_level;
        }

        self.level
    }

    pub fn release(&mut self) {
        self.target = 0.0;
        self.multiplier = self.release_multiplier;
    }

    pub fn attack(&mut self) {
        self.level += SILENCE + SILENCE;
        self.target = 2.0;
        self.multiplier = self.attack_multiplier;
    }

    #[inline(always)]
    pub fn is_active(&self) -> bool {
        self.level > SILENCE
    }

    /* NOTE: Not used yet
    #[inline(always)]
    pub fn is_in_attack(&self) -> bool {
        self.target >= 2.0
    }
    */
}
