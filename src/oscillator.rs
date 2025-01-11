const PI_OVER_FOUR: f32 = std::f32::consts::PI / 4.0; //0.7853981633974483;

pub struct Oscillator {
    pub amplitude: f32,

    pub period: f32,

    /// How fast to move through the phase
    pub increment: f32,

    /// Expected to be a value 0..1 to account for phase wrapping
    /// It is a "modulo counter" aka "phasor"
    pub phase: f32,

    phase_max: f32,

    dc_offset: f32,

    sin0: f32,
    sin1: f32,
    dsin: f32,
}

impl Default for Oscillator {
    fn default() -> Self {
        Self {
            amplitude: 1.0,
            period: 0.0,
            increment: 0.0,
            phase: 0.0,
            phase_max: 0.0,
            dc_offset: 0.0,
            sin0: 0.0,
            sin1: 0.0,
            dsin: 0.0,
        }
    }
}

impl Oscillator {
    pub fn reset(&mut self) {
        self.increment = 0.0;
        self.dc_offset = 0.0;
        self.phase = 0.0;
        self.sin0 = 0.0;
        self.sin1 = 0.0;
        self.dsin = 0.0;
    }

    pub fn next_sample(&mut self) -> f32 {
        let output;
        self.phase += self.increment;

        if self.phase <= PI_OVER_FOUR {
            let half_period = self.period / 2.0;
            self.phase_max = (0.5 + half_period).floor() - 0.5;
            self.dc_offset = 0.5 * self.amplitude / self.phase_max;
            self.phase_max *= std::f32::consts::PI;
            self.increment = self.phase_max / half_period;
            self.phase = -self.phase;

            self.sin0 = self.amplitude * self.phase.sin();
            self.sin1 = self.amplitude * (self.phase - self.increment).sin();
            self.dsin = 2.0 * self.increment.cos();

            if self.phase * self.phase > 1e-9 {
                output = self.sin0 / self.phase;
            } else {
                output = self.amplitude;
            }
        } else {
            if self.phase > self.phase_max {
                self.phase = self.phase_max + self.phase_max - self.phase;
                self.increment = -self.increment;
            }

            let sinp = self.dsin * self.sin0 - self.sin1;
            self.sin1 = self.sin0;
            self.sin0 = sinp;
            output = sinp / self.phase;
        }

        output - self.dc_offset
    }
}
