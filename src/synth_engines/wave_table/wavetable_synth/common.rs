use super::config::{N_ENV, N_LFO, N_OSC, POLYPHONY};
// use midi_control::MidiNote;
use serde::{Deserialize, Serialize};

pub type MidiNote = u8;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct DataTable {
    pub osc: [f32; N_OSC],
    pub env: [f32; N_ENV],
    pub lfos: [f32; N_LFO],
    pub filter_1: f32,
    pub filter_2: f32,
    pub chorus: f32,
    pub reverb: f32,
    pub direct_out: f32,
    // pub notes: [Option<(MidiNote, u8)>; POLYPHONY],
    // pub note: Option<MidiNote>,
    pub note: [Option<(MidiNote, u8)>; POLYPHONY],
    pub osc_order: Vec<usize>,
    // pub freq: Option<f32>,
    // pub velocity: Vec<<u8>,
    pub pitch_bend: f32,
    pub mod_wheel: f32,
    pub macros: [f32; 4],
}

impl DataTable {
    pub fn get_entry(&self, src: &ModMatrixSrc) -> f32 {
        match src {
            ModMatrixSrc::Velocity => {
                if let Some(i) = self.osc_order.last() {
                    self.note[*i]
                        .map(|(_note, vel)| vel as f32 / 127.0)
                        .unwrap_or(0.0)
                } else {
                    0.0
                }
            }
            ModMatrixSrc::Gate => {
                // if self.note.is_some() {
                //     1.0
                // } else {
                //     0.0
                // }
                if !self.osc_order.is_empty() {
                    1.0
                } else {
                    0.0
                }
            }
            ModMatrixSrc::PitchWheel => self.pitch_bend,
            ModMatrixSrc::ModWheel => self.mod_wheel,
            ModMatrixSrc::Env(i) => self.env[*i],
            ModMatrixSrc::Lfo(i) => self.lfos[*i],
            ModMatrixSrc::Macro1 => self.macros[0],
            ModMatrixSrc::Macro2 => self.macros[1],
            ModMatrixSrc::Macro3 => self.macros[2],
            ModMatrixSrc::Macro4 => self.macros[3],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ModMatrixSrc {
    Velocity,
    Env(usize),
    Lfo(usize),
    Gate,
    Macro1,
    Macro2,
    Macro3,
    Macro4,
    ModWheel,
    PitchWheel,
}

// #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
// pub enum Osc {
//     Osc1,
//     Osc2,
//     Osc3,
// }

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum OscParam {
    Level,
    Tune,
    // Pan,
}

// #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
// pub enum Env {
//     Env1,
//     Env2,
//     Env3,
//     Env4,
//     Env5,
// }

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum EnvParam {
    Atk,
    Dcy,
    Sus,
    Rel,
}

// #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
// pub enum Lfo {
//     Lfo1,
//     Lfo2,
//     Lfo3,
//     Lfo4,
// }

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum LfoParam {
    Speed,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum LowPass {
    LP1,
    LP2,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum LowPassParam {
    Cutoff,
    Res,
    Mix,
    // KeyTrack,
    // Drive,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ModMatrixDest {
    ModMatrixEntryModAmt(usize),
    Osc {
        osc: usize,
        param: OscParam,
    },
    Env {
        env: usize,
        param: EnvParam,
    },
    Lfo {
        lfo: usize,
        param: LfoParam,
    },
    LowPass {
        low_pass: LowPass,
        param: LowPassParam,
    },
    // Effect { effect: Effect, Param: usize }
    SynthVolume,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModMatrixItem {
    pub src: ModMatrixSrc,
    pub dest: ModMatrixDest,
    pub amt: f32,
    pub bipolar: bool,
}
