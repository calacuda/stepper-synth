use super::{Effect, EffectParam};
use crate::{synth_engines::LfoInput, HashMap, KnobCtrl, SampleGen, SAMPLE_RATE};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use std::fmt::Display;
use strum::{EnumIter, IntoEnumIterator};

#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "stepper_synth_backend", get_all, eq, eq_int, hash, frozen)
)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
pub enum ChorusParam {
    Volume,
    Speed,
}

impl Display for ChorusParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Volume => write!(f, "Vol."),
            Self::Speed => write!(f, "Speed"),
        }
    }
}

impl TryFrom<f32> for ChorusParam {
    type Error = String;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        let value = value as usize;

        Ok(match value {
            _ if value == Self::Volume as usize => Self::Volume,
            _ if value == Self::Speed as usize => Self::Speed,
            _ => return Err(format!("{value} could not be turned into a reverb param")),
        })
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl ChorusParam {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{self}"))
    }
}

impl EffectParam for ChorusParam {}

#[derive(Debug, Clone, Copy)]
pub struct Chorus {
    pub size: usize,
    pub buff: [f32; SAMPLE_RATE as usize],
    pub insert_i: usize,
    pub get_i: usize,
    pub step: usize,
    pub volume: f32,
    pub speed: f32,
    pub input: f32,
    lfo_input: LfoInput,
}

impl Chorus {
    pub fn new() -> Self {
        Self {
            size: SAMPLE_RATE as usize,
            buff: [0.0; SAMPLE_RATE as usize],
            insert_i: 0,
            get_i: 0,
            step: (SAMPLE_RATE as f32 * (0.25 * 0.5)) as usize,
            volume: 0.75,
            speed: 0.25,
            input: 0.0,
            lfo_input: LfoInput::default(),
        }
    }

    pub fn get_sample(&mut self) -> f32 {
        let chorus = self.buff[self.get_i] + self.input;
        // self.buff[self.i ] = echo;
        // self.buff[(self.i + self.step) % self.size] = chorus * self.volume;
        // self.buff[self.i] = 0.0;
        // self.buff[(self.i as i64 - self.step as i64).abs() as usize % self.size] = echo;
        self.get_i = (self.get_i + self.step) % self.size;
        // if echo == input_sample && input_sample != 0.0 {
        //     error!("[error] {}", self.i);
        // }
        chorus.tanh()
    }

    /// sets speed, takes speed in seconds
    pub fn set_speed(&mut self, speed: f32) {
        // info!("speed: {}", speed);
        self.speed = speed;
        // self.step = (SAMPLE_RATE as f32 * (speed * 0.05)) as usize;
        self.step = (SAMPLE_RATE as f32 * (speed * 0.5)) as usize;
        // info!("step:  {}", self.step);
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

impl SampleGen for Chorus {
    fn get_sample(&mut self) -> f32 {
        self.get_sample()
    }
}

impl KnobCtrl for Chorus {
    fn get_lfo_input(&mut self) -> &mut LfoInput {
        &mut self.lfo_input
    }
}

impl Effect for Chorus {
    fn take_input(&mut self, value: f32) {
        self.input = value * self.volume;
        self.buff[self.insert_i] = self.input;
        self.insert_i += 1;
        self.insert_i %= self.size;
    }

    fn get_param_list(&self) -> Vec<String> {
        ChorusParam::iter()
            .map(|param| format!("{param}"))
            .collect()
    }

    fn get_params(&self) -> crate::HashMap<String, f32> {
        let map = HashMap::default();

        // TODO: Write this

        map
    }

    fn set_param(&mut self, _param: &str, _to: f32) {
        // TODO: Write this
    }
}
