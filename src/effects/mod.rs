use crate::SampleGen;

pub mod chorus;
pub mod reverb;

pub trait Effect: SampleGen + Send {
    fn take_input(&mut self, value: f32);
}
