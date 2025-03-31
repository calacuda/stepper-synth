use super::{
    calculate_modulation,
    common::{DataTable, LowPass as LP, ModMatrixDest},
    config::{N_ENV, N_OSC},
    // effects::{chorus::Chorus, /* reverb::Reverb, */ Effect, EffectsModule},
    // lfo::LFO,
    midi_to_freq,
    synth_engines::{
        synth::osc::{OscTarget, Oscillator},
        synth_common::{env::ADSR, moog_filter::LowPass},
    },
    ModMatrix,
    ModulationDest,
    OscWaveTable,
    // SampleGen,
};
// use crate::effects::{chorus::Chorus, /* reverb::Reverb, */ Effect, EffectsModule};
use array_macro::array;
use core::ops::IndexMut;
use log::{info, warn};

const U32: f32 = u32::MAX as f32 * 0.5;

// #[macro_export]
// macro_rules! array {
//     [$expr:expr; 0] => {
//         []
//     };
//     // [$expr:expr; $count:expr, false] =>
//     // };
//     [$expr:expr; $count:expr] => {{
//         // let value = $expr;
//         // $crate::array![_ => $crate::__core::clone::Clone::clone(&value); $count]
//         [ $crate::array![$expr; $count - 1, false], $expr ]
//     }};
//     // [$i:pat => $e:expr; $count:expr] => {
//     //     $crate::__array![$i => $e; $count]
//     // };
// }

#[derive(Clone, Debug)]
pub struct Voice {
    /// Oscilators holds the osc and if its playing
    pub oscs: [(Oscillator, bool); N_OSC],
    /// env filters
    pub envs: [ADSR; N_ENV],
    // /// LFOs
    // pub lfos: [LFO; N_LFO],
    /// filters
    pub filters: [LowPass; 2],
    /// what notes this voice is playing
    pub playing: Option<u8>,
    // /// effects, holds the effect and if its one or not
    // pub effects: [(EffectsModule, bool); 1],
    /// describes how loud the synth is
    pub level: f32,
    /// stores the modulation amount for level
    level_mod: f32,
}

impl Voice {
    pub fn new(wave_table: OscWaveTable) -> Self {
        // let effects = [
        //     (EffectsModule::Chorus(Chorus::new()), false),
        //     // (EffectsModule::Reverb(Reverb::new()), false),
        // ];
        // let lpf = LowPass::new();
        let mut oscs = array![(Oscillator::new(wave_table), false); N_OSC];
        oscs[0].1 = true;
        oscs[1].1 = true;
        oscs[1].0.offset = -24;
        // oscs[2].1 = true;
        let targets = [
            OscTarget::Filter1,
            OscTarget::Filter2,
            OscTarget::Filter1_2,
            // OscTarget::Effects,
            OscTarget::DirectOut,
            // OscTarget::DirectOut,
            // OscTarget::DirectOut,
            // OscTarget::DirectOut,
            // OscTarget::DirectOut,
        ];

        for i in 0..N_OSC {
            oscs.index_mut(i).0.target = targets[i];
            // info!("{:?}", targets[i]);
            info!("{:?}", oscs.index_mut(i).0.target);
        }

        // info!("{oscs:?}");

        Self {
            oscs,
            envs: array![ADSR::new(); N_ENV],
            // lfos: array![LFO::new(); N_LFO],
            filters: [LowPass::new(), LowPass::new()],
            playing: None,
            // effects,
            level: 1.0,
            level_mod: 0.0,
        }
    }

    pub fn press(&mut self, midi_note: u8, _velocity: u8) {
        // info!("velocity => {velocity}");
        self.oscs.iter_mut().for_each(|osc| {
            if osc.1 {
                osc.0.press(midi_note)
            }
        });
        self.envs.iter_mut().for_each(|env| env.press());
        self.filters.iter_mut().for_each(|filter| {
            if filter.key_track {
                filter.set_note(midi_to_freq(midi_note as i16));
            }
        });
        // self.lfos.iter_mut().for_each(|lfo| lfo.press());
        self.playing = Some(midi_note);
        // self.data_table.velocity = Some(velocity);
        // self.data_table.note = Some(midi_note);
    }

    pub fn release(&mut self) {
        self.oscs.iter_mut().for_each(|osc| {
            if osc.1 {
                osc.0.release()
            }
        });
        // self.lfos.iter_mut().for_each(|lfo| lfo.release());
        self.envs.iter_mut().for_each(|env| env.release());
        // self.filters.iter_mut().for_each(|filter| {
        //     if filter.key_track {
        //         filter.set_note(midi_to_freq(midi_note));
        //     }
        // });
        // self.lfos.iter_mut().for_each(|lfo| lfo.start());
    }

    /// resets the mod matrix along with the effects, lfos, oscilators, etc
    pub fn reset(&mut self) {
        // self.lfos.iter_mut().for_each(|lfo| lfo.index);
        self.oscs.iter_mut().for_each(|(osc, on)| {
            if *on {
                osc.reset()
            }
        });
        // self.lfos.iter_mut().for_each(|lfo| lfo.reset());
        self.envs.iter_mut().for_each(|env| env.reset());
        self.filters.iter_mut().for_each(|lp| lp.reset());
        // TODO: figure out how to reset a modulation of mod amount

        // reset self
        self.level_mod = 0.0;
    }

    /// send data from data_table where ever it needs to go, based on the mod_natrix
    pub fn route_mod_matrix(&mut self, mod_matrix: &ModMatrix, data_table: &mut DataTable) {
        // info!("{:?}", self.data_table.velocity);

        for mod_entry in mod_matrix {
            if let Some(entry) = mod_entry {
                // get mod amount
                let mut amt = data_table.get_entry(&entry.src) * entry.amt;

                if entry.bipolar {
                    amt -= entry.amt / 2.0;
                }

                // info!("src {:?}, amt {}, dest {:?}", entry.src, amt, entry.dest);

                match entry.dest {
                    ModMatrixDest::ModMatrixEntryModAmt(mod_amt_amt) => {
                        todo!("figure out a way to modulate mod amounts");
                    }
                    ModMatrixDest::Osc { osc, param } => {
                        let (osc, on) = &mut self.oscs[osc];

                        if *on {
                            osc.modulate(param, amt);
                        }
                    }
                    ModMatrixDest::Env { env, param } => self.envs[env].modulate(param, amt),
                    // ModMatrixDest::Lfo { lfo, param } => self.lfos[lfo].modulate(param, amt),
                    ModMatrixDest::Lfo { lfo: _, param: _ } => {}
                    ModMatrixDest::LowPass { low_pass, param } => match low_pass {
                        LP::LP1 => self.filters[0].modulate(param, amt),
                        LP::LP2 => self.filters[1].modulate(param, amt),
                    },
                    ModMatrixDest::SynthVolume => self.modulate_level(amt),
                };
            }
        }
        // for (osc, on) in self.oscs.iter_mut() {
        //     if on {
        //         osc.mod
        //     }
        // }
    }

    fn modulate_level(&mut self, amt: f32) {
        self.level_mod = amt;
    }

    pub fn get_sample(
        &mut self,
        mod_matrix: &ModMatrix,
        data_table: &mut DataTable,
    ) -> Option<f32> {
        if self.playing.is_none() {
            return None;
        }

        self.route_mod_matrix(mod_matrix, data_table);

        // calculate envs
        for (i, env) in self.envs.iter_mut().enumerate() {
            let sample = env.get_samnple();
            data_table.env[i] = sample;
        }

        // // calculate lfos
        // for (i, lfo) in self.lfos.iter_mut().enumerate() {
        //     let sample = lfo.get_sample();
        //     data_table.lfos[i] = sample;
        // }

        if !self.envs[0].pressed() && data_table.env[0] <= 0.0 {
            self.playing = None;
            self.reset();
            return None;
        }

        let mut output = 0.0;

        let mut osc_sample = 0.0;

        for (i, (osc, on)) in self.oscs.iter_mut().enumerate() {
            if *on {
                // continue;

                let sample = osc.get_sample();

                data_table.osc[i] = sample;

                // osc_sample += // self.filters[0].get_sample(sample);
                osc_sample += match osc.target {
                    OscTarget::Filter1 => self.filters[0].get_sample(sample),
                    OscTarget::Filter2 => self.filters[1].get_sample(sample),
                    OscTarget::Filter1_2 => {
                        self.filters[0].get_sample(sample) + self.filters[1].get_sample(sample)
                    }
                    // OscTarget::Effects => sample,
                    OscTarget::DirectOut => {
                        output += sample;

                        continue;
                    }
                };
            }
        }

        // let mut effects_sample = osc_sample;

        // for (effect, on) in self.effects.iter_mut() {
        //     if *on {
        //         effect.take_input(effects_sample);
        //
        //         effects_sample += effect.get_sample();
        //     }
        // }

        // output += effects_sample;
        output += osc_sample;

        // TODO: add an allpass filter.
        let sample = output * data_table.env[0] * calculate_modulation(self.level, self.level_mod);
        // warn!("{}", get_u32_sample(Some(sample)));

        Some(sample)
    }
}

pub fn get_u32_sample(sample: Option<f32>) -> u32 {
    let sample = sample.unwrap();
    warn!("{sample}");
    // let sample = (u32::MAX as f64 * (sample * 0.5 + 0.5)).round() as u32;
    // warn!("{sample}");

    // sample
    let normalized = (sample + 1.0) * U32;
    let converted = normalized as u32;

    converted
}
