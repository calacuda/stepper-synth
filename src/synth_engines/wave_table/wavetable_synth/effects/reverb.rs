use super::{Effect, EffectParam};
use crate::SampleGen;
use core::fmt::Display;
use reverb;
use strum::EnumIter;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
pub enum ReverbParam {
    Gain,
    Decay,
    Damping,
    Cutoff,
}

// impl TryFrom<f32> for ReverbParam {
//     type Error = String;
//
//     fn try_from(value: f32) -> Result<Self, Self::Error> {
//         let value = value as usize;
//
//         Ok(match value {
//             _ if value == Self::Gain as usize => Self::Gain,
//             _ if value == Self::Decay as usize => Self::Decay,
//             _ if value == Self::Damping as usize => Self::Damping,
//             _ if value == Self::Cutoff as usize => Self::Cutoff,
//             _ => return Err(format!("{value} could not be turned into a reverb param")),
//         })
//     }
// }

impl Display for ReverbParam {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::Gain => write!(f, "Gain"),
            Self::Decay => write!(f, "Decay"),
            Self::Damping => write!(f, "Damping"),
            Self::Cutoff => write!(f, "Cutoff"),
        }
    }
}

impl EffectParam for ReverbParam {}

// impl FromStr for ReverbParam {
//     type Err = String;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.to_lowercase().as_str() {
//             "gain" => Ok(Self::Gain),
//             "decay" => Ok(Self::Decay),
//             "damping" => Ok(Self::Damping),
//             "cutoff" => Ok(Self::Cutoff),
//             _ => Err(format!("unknown reverb param {s}")),
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct Reverb {
    pub effect: reverb::Reverb,
    pub gain: f32,
    pub decay: f32,
    in_sample: f32,
    damping: f32,
    cutoff: f32,
    // lfo_sample: f32,
    // lfo_target: Option<ReverbParam>,
    // lfo_input: LfoInput,
}

impl Default for Reverb {
    fn default() -> Self {
        Self::new()
    }
}

impl Reverb {
    pub fn new() -> Self {
        Self {
            effect: reverb::Reverb::new(),
            gain: 0.75,
            decay: 0.5,
            in_sample: 0.0,
            damping: 0.0,
            cutoff: 1.0,
            // lfo_sample: 0.0,
            // lfo_target: None,
            // lfo_input: LfoInput::default(),
        }
    }

    pub fn get_sample(&mut self, in_sample: f32) -> f32 {
        let gain = self.gain;

        self.lfo_step();

        self.effect.calc_sample(in_sample, gain)
    }

    /// apply lfo to controls
    fn lfo_step(&mut self) {
        // TODO: Write this
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay;

        self.effect = self.effect.decay(decay).clone();
    }

    pub fn set_damping(&mut self, value: f32) {
        self.damping = value;

        self.effect = self.effect.damping(value).clone();
    }

    pub fn set_cutoff(&mut self, value: f32) {
        self.cutoff = value;

        self.effect = self.effect.bandwidth(value).clone();
    }
}

impl SampleGen for Reverb {
    fn get_sample(&mut self) -> f32 {
        self.get_sample(self.in_sample)
    }
}

// impl KnobCtrl for Reverb {
//     fn knob_1(&mut self, value: f32) -> bool {
//         // info!("setting gain");
//         self.set_gain(value);
//
//         true
//     }
//
//     fn knob_2(&mut self, value: f32) -> bool {
//         // info!("setting decay");
//         self.set_decay(value);
//
//         true
//     }
//
//     fn knob_3(&mut self, value: f32) -> bool {
//         // info!("setting Damping");
//         self.set_damping(value);
//
//         true
//     }
//
//     fn knob_4(&mut self, value: f32) -> bool {
//         // info!("setting cutoff");
//         self.set_cutoff(value);
//
//         true
//     }
// }

impl Effect for Reverb {
    // type Param = ReverbParam;

    fn take_input(&mut self, value: f32) {
        self.in_sample = value;
    }

    // fn get_param_list(&self) -> Vec<String> {
    //     ReverbParam::iter()
    //         .map(|param| format!("{param}"))
    //         .collect()
    // }
    //
    // fn get_params(&self) -> crate::HashMap<String, f32> {
    //     let mut map = HashMap::default();
    //
    //     map.insert("Gain".into(), self.gain);
    //     map.insert("Decay".into(), self.decay);
    //     map.insert("Damping".into(), self.damping);
    //     map.insert("Cutoff".into(), self.cutoff);
    //
    //     map
    // }
    //
    // fn set_param(&mut self, param: &str, to: f32) {
    //     let Ok(param) = ReverbParam::from_str(param) else {
    //         return;
    //     };
    //
    //     match param {
    //         ReverbParam::Gain => self.set_gain(to),
    //         ReverbParam::Decay => self.set_decay(to),
    //         ReverbParam::Cutoff => self.set_cutoff(to),
    //         ReverbParam::Damping => self.set_damping(to),
    //     }
    // }

    // fn set_param(&mut self, param: Self::Param, to: f32) {
    //     match param {
    //         ReverbParam::Gain => self.set_gain(to),
    //         ReverbParam::Decay => self.set_decay(to),
    //         ReverbParam::Cutoff => self.set_cutoff(to),
    //         ReverbParam::Damping => self.set_damping(to),
    //     }
    // }

    // fn get_param_value(&self, param: Self::Param) -> f32 {
    //     match param {
    //         ReverbParam::Gain => self.gain,
    //         ReverbParam::Decay => self.decay,
    //         ReverbParam::Cutoff => self.cutoff,
    //         ReverbParam::Damping => self.damping,
    //     }
    // }

    // fn lfo_nudge_param(&mut self, param: Self::Param) {
    //     self.effect = match param {
    //         ReverbParam::Gain => return,
    //         ReverbParam::Decay => self.effect.decay(self.decay * self.lfo_sample).clone(),
    //         ReverbParam::Damping => self.effect.damping(self.damping * self.lfo_sample).clone(),
    //         ReverbParam::Cutoff => self.effect.bandwidth(self.cutoff * self.lfo_sample).clone(),
    //     }
    // }
}
