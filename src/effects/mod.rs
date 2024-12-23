use crate::{synth_engines::Param, HashMap, KnobCtrl, SampleGen};
use chorus::Chorus;
use enum_dispatch::enum_dispatch;
use pyo3::{prelude::*, PyClass};
use reverb::Reverb;
use std::fmt::{Debug, Display};

pub mod chorus;
pub mod reverb;

#[pyclass(module = "stepper_synth_backend", get_all, eq, eq_int, hash, frozen)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectType {
    Reverb,
    Chorus,
    Delay,
}

impl Display for EffectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Reverb => write!(f, "Reverb"),
            Self::Chorus => write!(f, "Chorus"),
            Self::Delay => write!(f, "Delay"),
        }
    }
}

#[pymethods]
impl EffectType {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{self}"))
    }
}

pub trait EffectParam: Debug + Clone + Display + TryFrom<f32> + PyClass {}

// TODO: use this enum dispatch trick on the synth_engines too.
#[enum_dispatch(EffectsModule)]
pub trait Effect: Debug + SampleGen + Send + KnobCtrl {
    // type Param: EffectParam;

    fn take_input(&mut self, value: f32);
    fn get_param_list(&self) -> Vec<String>;
    fn get_params(&self) -> HashMap<String, f32>;
    // fn set_param(&mut self, param: Self::Param, to: f32);
    // fn nudge_param(&mut self, param: Self::Param, by: f32) {
    //     self.set_param(param.clone(), self.get_param_value(param) + by);
    // }
    // fn lfo_nudge_param(&mut self, param: Self::Param);
    //
    // fn get_param_value(&self, param: ) -> f32;
}

#[derive(Debug)]
pub struct EffectsModules {
    pub effect: EffectType,
    effect_i: usize,
    effects: Vec<EffectsModule>,
}

impl EffectsModules {
    pub fn new() -> Self {
        Self {
            effect: EffectType::Reverb,
            effect_i: 0,
            effects: vec![
                EffectsModule::Reverb(Reverb::new()),
                EffectsModule::Chorus(Chorus::new()),
            ],
        }
    }

    pub fn set(&mut self, new_type: EffectType) {
        self.effect_i = match new_type {
            EffectType::Reverb => 0,
            EffectType::Chorus => 1,
            EffectType::Delay => return,
        };

        self.effect = new_type;
    }

    pub fn take_input(&mut self, sample: f32) {
        if let Some(effect) = self.effects.get_mut(self.effect_i) {
            effect.take_input(sample)
        }
    }

    pub fn get_params(&self) -> HashMap<String, f32> {
        if let Some(effect) = self.effects.get(self.effect_i) {
            effect.get_params()
        } else {
            HashMap::default()
        }
    }

    pub fn lfo_control(&mut self, param: Param, lfo_sample: f32) {
        if let Some(effect) = self.effects.get_mut(self.effect_i) {
            effect.lfo_control(param, lfo_sample)
        }
    }
}

impl SampleGen for EffectsModules {
    fn get_sample(&mut self) -> f32 {
        if let Some(effect) = self.effects.get_mut(self.effect_i) {
            effect.get_sample()
        } else {
            0.0
        }
    }
}

#[enum_dispatch]
#[derive(Debug, Clone)]
pub enum EffectsModule {
    Reverb(Reverb),
    Chorus(Chorus),
}

impl EffectsModule {
    // pub fn new() -> Self {
    //     Self::Reverb(Reverb::new())
    // }

    // fn do_get_param(effect: &impl Effect) -> HashMap<String, f32> {
    //     effect
    //         .get_param_list()
    //         .into_iter()
    //         .map(|param| (format!("{param}"), effect.get_param_value(param.clone())))
    //         .collect()
    // }

    // pub fn get_params(&self) -> HashMap<String, f32> {
    //     match self {
    //         Self::Reverb(effect) => Self::do_get_param(effect),
    //         Self::Chorus(effect) => Self::do_get_param(effect),
    //     }
    // }

    // fn do_lfo_set_param(effect: &mut impl Effect, param_n: usize, to: f32) {
    //     let params = effect.get_param_list();
    //
    //     let Some(param) = params.get(param_n) else {
    //         return;
    //     };
    //
    //     effect.lfo_control(param.clone(), to);
    // }
}

// impl From<EffectType> for EffectsModules {
//     fn from(value: EffectType) -> Self {
//         match value {
//             EffectType::Reverb => Self::Reverb(Reverb::new()),
//             EffectType::Chorus => Self::Chorus(Chorus::new()),
//             // TODO: write delay
//             EffectType::Delay => Self::Chorus(Chorus::new()),
//         }
//     }
// }

impl KnobCtrl for EffectsModules {
    fn knob_1(&mut self, value: f32) -> bool {
        if let Some(effect) = self.effects.get_mut(self.effect_i) {
            // info!("setting knob 1 for {effect:?}");
            effect.knob_1(value)
        } else {
            false
        }
    }

    fn knob_2(&mut self, value: f32) -> bool {
        if let Some(effect) = self.effects.get_mut(self.effect_i) {
            // info!("setting knob 2 for {effect:?}");
            effect.knob_2(value)
        } else {
            false
        }
    }

    fn knob_3(&mut self, value: f32) -> bool {
        if let Some(effect) = self.effects.get_mut(self.effect_i) {
            // info!("setting knob 3 for {effect:?}");
            effect.knob_3(value)
        } else {
            false
        }
    }

    fn knob_4(&mut self, value: f32) -> bool {
        if let Some(effect) = self.effects.get_mut(self.effect_i) {
            // info!("setting knob 4 for {effect:?}");
            effect.knob_4(value)
        } else {
            false
        }
    }

    fn lfo_control(&mut self, param: crate::synth_engines::Param, lfo_sample: f32) {
        // match self {
        //     Self::Reverb(effect) => effect.lfo_control(param, lfo_sample),
        //     Self::Chorus(effect) => effect.lfo_control(param, lfo_sample),
        // }
        if let Some(ref mut effect) = self.effects.get_mut(self.effect_i) {
            effect.lfo_control(param, lfo_sample)
        }
    }
}
