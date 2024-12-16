use crate::SampleGen;
use std::fmt::Debug;

pub mod chorus;
pub mod reverb;

pub trait Effect: Debug + SampleGen + Send {
    fn take_input(&mut self, value: f32);
}
