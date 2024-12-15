use crate::SAMPLE_RATE;
use num_traits::clamp;
use std::f32::consts::PI;

// Moog filter from
// https://github.com/ddiakopoulos/MoogLadders
// (LGPLv3)

#[derive(Clone, Copy, Debug)]
pub struct HuovilainenMoog {
    stage: [f32; 4],
    stage_tanh: [f32; 3],
    delay: [f32; 6],
    tune: f32,
    acr: f32,
    res_quad: f32,
    coeff_cutoff: f32,
    coeff_resonance: f32,
    sample_rate: f32,
}

const THERMAL: f32 = 0.000025f32;

impl HuovilainenMoog {
    pub fn new() -> Self {
        let mut filter = Self {
            stage: [0.0; 4],
            stage_tanh: [0.0; 3],
            delay: [0.0; 6],
            tune: 0.0,
            acr: 0.0,
            res_quad: 0.0,
            coeff_cutoff: 0.0,
            coeff_resonance: 0.0,
            sample_rate: SAMPLE_RATE as f32,
        };

        filter.compute_coeffs(5_000.0, 0.75);

        filter
    }

    fn compute_coeffs(&mut self, cutoff: f32, resonance: f32) {
        if self.coeff_cutoff == cutoff && self.coeff_resonance == resonance {
            return;
        }

        let total_cutoff = clamp(cutoff, 0.0, self.sample_rate / 2.0);

        let fc = total_cutoff / self.sample_rate;
        let f = fc * 0.5; // oversampled
        let fc2 = fc * fc;
        let fc3 = fc * fc * fc;

        let fcr = 1.8730 * fc3 + 0.4955 * fc2 - 0.6490 * fc + 0.9988;
        self.acr = -3.9364 * fc2 + 1.8409 * fc + 0.9968;

        self.tune = (1.0 - (-((2.0 * PI) * f * fcr)).exp()) / THERMAL;

        self.res_quad = 4.0 * resonance * self.acr;

        // Cache the coeffs for the
        self.coeff_cutoff = cutoff;
        self.coeff_resonance = resonance;
    }

    pub fn process(
        &mut self,
        in_sample: f32,
        // sample_rate: f32,
        cutoff: f32,
        resonance: f32,
    ) -> f32 {
        self.compute_coeffs(cutoff, resonance);

        // Oversample
        for _j in 0..2 {
            let input = in_sample - self.res_quad * self.delay[5];
            self.stage[0] =
                self.delay[0] + self.tune * (tanh(input * THERMAL) - self.stage_tanh[0]);
            self.delay[0] = self.stage[0];
            for k in 1..4 {
                let input = self.stage[k - 1];
                self.stage_tanh[k - 1] = tanh(input * THERMAL);
                self.stage[k] = self.delay[k]
                    + self.tune
                        * (self.stage_tanh[k - 1]
                            - (if k != 3 {
                                self.stage_tanh[k]
                            } else {
                                tanh(self.delay[k] * THERMAL)
                            }));
                self.delay[k] = self.stage[k];
            }
            // 0.5 sample delay for phase compensation
            self.delay[5] = (self.stage[3] + self.delay[4]) * 0.5;
            self.delay[4] = self.stage[3];
        }
        self.delay[5] as f32
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LowPass {
    filter: HuovilainenMoog,
    cutoff: f32,
    resonance: f32,
}

impl LowPass {
    pub fn new() -> Self {
        let mut filter = HuovilainenMoog::new();
        filter.compute_coeffs(5_000.0, 0.75);

        Self {
            filter,
            cutoff: 5_000.0,
            resonance: 0.75,
        }
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
    }

    pub fn set_resonace(&mut self, res: f32) {
        self.resonance = res;
    }

    pub fn get_sample(&mut self, sample: f32, env: f32) -> f32 {
        self.filter
            .process(sample, self.cutoff * env, self.resonance * env)
    }
}

#[inline]
fn tanh(x: f32) -> f32 {
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;

    let a = x + (0.16489087 * x3) + (0.00985468 * x5);

    a / (1.0 + (a * a)).sqrt()
}
