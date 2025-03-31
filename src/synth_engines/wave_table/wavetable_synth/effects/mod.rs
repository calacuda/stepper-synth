use crate::SampleGen;
use chorus::Chorus;
use core::fmt::{Debug, Display};
use enum_dispatch::enum_dispatch;
// use reverb::Reverb;
use strum::EnumIter;

pub mod chorus;
// pub mod reverb;

pub trait EffectParam: Debug + Clone + Display /* + TryFrom<f32> */ {}

#[enum_dispatch(EffectsModule)]
pub trait Effect: Debug + SampleGen + Send {
    // type Param: EffectParam;

    fn take_input(&mut self, value: f32);
    // fn get_param_list(&self) -> Vec<String>;
    // fn get_params(&self) -> HashMap<String, f32>;
    // fn get_params(&self) -> HashMap<String, f32>;
    // fn set_param(&mut self, param: &str, to: f32);
}

#[enum_dispatch]
#[derive(Debug, Clone, EnumIter)]
pub enum EffectsModule {
    // Reverb(Reverb),
    Chorus(Chorus),
}

// impl From<EffectType> for EffectsModule {
//     fn from(value: EffectType) -> Self {
//         match value {
//             EffectType::Reverb => Self::Reverb(Reverb::new()),
//             EffectType::Chorus => Self::Chorus(Chorus::new()),
//         }
//     }
// }
