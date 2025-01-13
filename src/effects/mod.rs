use crate::{HashMap, KnobCtrl, SampleGen};
use chorus::Chorus;
use enum_dispatch::enum_dispatch;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use reverb::Reverb;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use strum::EnumIter;

pub mod chorus;
pub mod reverb;

#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "stepper_synth_backend", get_all, eq, eq_int, hash, frozen)
)]
#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, EnumIter, Serialize, Deserialize,
)]
pub enum EffectType {
    Reverb,
    Chorus,
    // Delay,
}

impl Into<usize> for EffectType {
    fn into(self) -> usize {
        match self {
            Self::Reverb => 0,
            Self::Chorus => 1,
            // Self::Delay => 2,
        }
    }
}

impl Display for EffectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Reverb => write!(f, "Reverb"),
            Self::Chorus => write!(f, "Chorus"),
            // Self::Delay => write!(f, "Delay"),
        }
    }
}

#[cfg(feature = "pyo3")]
#[cfg_attr(feature = "pyo3", pymethods)]
impl EffectType {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{self}"))
    }
}

pub trait EffectParam: Debug + Clone + Display + TryFrom<f32> {}

#[enum_dispatch(EffectsModule)]
pub trait Effect: Debug + SampleGen + Send + KnobCtrl {
    // type Param: EffectParam;

    fn take_input(&mut self, value: f32);
    fn get_param_list(&self) -> Vec<String>;
    fn get_params(&self) -> HashMap<String, f32>;
    fn set_param(&mut self, param: &str, to: f32);
}

#[enum_dispatch]
#[derive(Debug, Clone)]
pub enum EffectsModule {
    Reverb(Reverb),
    Chorus(Chorus),
}

impl From<EffectType> for EffectsModule {
    fn from(value: EffectType) -> Self {
        match value {
            EffectType::Reverb => Self::Reverb(Reverb::new()),
            EffectType::Chorus => Self::Chorus(Chorus::new()),
        }
    }
}
