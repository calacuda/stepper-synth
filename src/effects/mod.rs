use crate::{HashMap, SampleGen};
use chorus::Chorus;
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

pub trait Effect: Debug + SampleGen + Send {
    type Param: EffectParam;

    fn take_input(&mut self, value: f32);
    fn get_param_list(&self) -> Vec<Self::Param>;
    fn set_param(&mut self, param: Self::Param, to: f32);
    fn nudge_param(&mut self, param: Self::Param, by: f32) {
        self.set_param(param.clone(), self.get_param_value(param) + by);
    }
    fn get_param_value(&self, param: Self::Param) -> f32;
}

#[derive(Debug, Clone)]
pub enum EffectsModule {
    Reverb(Reverb),
    Chorus(Chorus),
}

impl EffectsModule {
    pub fn new() -> Self {
        Self::Reverb(Reverb::new())
    }

    fn do_get_param(effect: &impl Effect) -> HashMap<String, f32> {
        effect
            .get_param_list()
            .into_iter()
            .map(|param| (format!("{param}"), effect.get_param_value(param.clone())))
            .collect()
    }

    pub fn get_params(&self) -> HashMap<String, f32> {
        match self {
            Self::Reverb(effect) => Self::do_get_param(effect),
            Self::Chorus(effect) => Self::do_get_param(effect),
        }
    }
}

impl From<EffectType> for EffectsModule {
    fn from(value: EffectType) -> Self {
        match value {
            EffectType::Reverb => Self::Reverb(Reverb::new()),
            EffectType::Chorus => Self::Chorus(Chorus::new()),
            // TODO: write delay
            EffectType::Delay => Self::Chorus(Chorus::new()),
        }
    }
}
