use crate::synth_engines::wave_table::wavetable_synth::{
    calculate_modulation, common::LowPassParam, config::SAMPLE_RATE, exp, tanh, ModulationDest,
};
use core::f32::consts::PI;
// use libm::{expf, tanhf};
#[allow(unused_imports)]
use log::*;

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

        filter.compute_coeffs(5_000.0, 0.5);

        filter
    }

    fn compute_coeffs(&mut self, cutoff: f32, resonance: f32) {
        if self.coeff_cutoff == cutoff && self.coeff_resonance == resonance {
            return;
        }

        let total_cutoff = cutoff.clamp(0.0, self.sample_rate / 2.0);

        let fc = total_cutoff / self.sample_rate;
        let f = fc * 0.5; // oversampled
        let fc2 = fc * fc;
        let fc3 = fc * fc * fc;

        let fcr = 1.8730 * fc3 + 0.4955 * fc2 - 0.6490 * fc + 0.9988;
        self.acr = -3.9364 * fc2 + 1.8409 * fc + 0.9968;
        // let exponent =;

        self.tune = (1.0 - exp(-1.0 * ((2.0 * PI) * f * fcr))) / THERMAL;

        // warn!(
        //     "{exp} => {} {} {}",
        //     exp.exp(),
        //     expf(exp),
        //     exp.exp() == expf(exp)
        // );
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
        // warn!("in_sample {in_sample}");
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
    pub cutoff: f32,
    pub resonance: f32,
    pub mix: f32,
    pub note: f32,
    // pub range: (f32, f32),
    pub key_track: bool,
    pub cutoff_mod: f32,
    pub res_mod: f32,
    pub mix_mod: f32,
}

impl LowPass {
    pub fn new() -> Self {
        let filter = HuovilainenMoog::new();
        // filter.compute_coeffs(5_000.0, 0.75);

        Self {
            filter,
            cutoff: 0.5,
            resonance: 0.5,
            note: 0.0,
            // range: (0.0, 0.0),
            key_track: true,
            mix: 0.25,
            cutoff_mod: 0.0,
            res_mod: 0.0,
            mix_mod: 0.0,
        }
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
    }

    pub fn set_resonace(&mut self, res: f32) {
        self.resonance = res;
    }

    pub fn get_sample(&mut self, sample: f32) -> f32 {
        // let nudge = 2.0_f32.powf(19.0 * env * self.cutoff / 12.0);
        // // let delta = (self.note * 2.0) - (self.note / 2.0);
        // // let nudge = delta * env * self.cutoff;
        // let cutoff = if env > 0.1 {
        //     self.note * nudge
        // } else if nudge < 0.1 {
        //     self.note / nudge
        // } else {
        //     self.note
        // };
        let delta = self.note * 21.0;
        let nudge = delta * calculate_modulation(self.cutoff, self.cutoff_mod);
        let cutoff = (self.note) + nudge;

        // warn!("res {}", calculate_modulation(self.resonance, self.res_mod));
        let mix = calculate_modulation(self.mix, self.mix_mod);

        self.filter.process(
            sample,
            cutoff,
            calculate_modulation(self.resonance, self.res_mod),
        ) * (1.0 - mix)
            + sample * mix
    }

    pub fn set_note(&mut self, note: f32) {
        self.note = note;
    }
}

impl ModulationDest for LowPass {
    type ModTarget = LowPassParam;

    fn modulate(&mut self, what: Self::ModTarget, by: f32) {
        match what {
            Self::ModTarget::Cutoff => self.cutoff_mod = by,
            Self::ModTarget::Res => self.res_mod = by,
            Self::ModTarget::Mix => self.mix_mod = by,
        }
    }

    fn reset(&mut self) {
        self.cutoff_mod = 0.0;
        self.res_mod = 0.0;
        self.mix_mod = 0.0;
    }
}
