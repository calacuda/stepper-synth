use crate::{
    effects::{Effect, EffectType, EffectsModule, EffectsModules},
    pygame_coms::{GuiParam, Knob, SynthEngineType},
    HashMap, KnobCtrl, SampleGen,
};
use midi_control::MidiNote;
use organ::organ::Organ;
use pyo3::prelude::*;
use std::fmt::Debug;
use synth_common::lfo::LFO;

pub mod organ;
pub mod synth;
pub mod synth_common;

pub trait SynthEngine: Debug + SampleGen + KnobCtrl + Send {
    fn name(&self) -> String;

    fn play(&mut self, note: MidiNote, velocity: u8);
    fn stop(&mut self, note: MidiNote);
    fn bend(&mut self, amount: f32);
    fn unbend(&mut self) {
        self.bend(0.0);
    }
    fn volume_swell(&mut self, amount: f32) -> bool;
    fn get_params(&self) -> HashMap<Knob, f32>;
    fn get_gui_params(&self) -> HashMap<GuiParam, f32>;
}

#[pyclass(module = "stepper_synth_backend", get_all, eq)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Param {
    Knob(Knob),
    Gui(GuiParam),
}

#[pyclass(module = "stepper_synth_backend", get_all, eq)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum LfoTarget {
    Synth(Param),
    Effect(Param),
}
#[derive(Debug)]
pub struct Synth {
    pub lfo: LFO,
    pub engine: Box<dyn SynthEngine>,
    pub engine_type: SynthEngineType,
    // pub effect: EffectType,
    pub effect_power: bool,
    pub effect: EffectsModules,
    // pub effect_power: bool,
    pub lfo_target: Option<LfoTarget>,
    pub lfo_routed: bool,
}

impl Synth {
    pub fn new() -> Self {
        Self {
            lfo: LFO::new(),
            effect: EffectsModules::new(),
            // effect: EffectType::Reverb,
            effect_power: false,
            lfo_target: None,
            engine_type: SynthEngineType::B3Organ,
            engine: Box::new(Organ::new()),
            lfo_routed: false,
        }
    }

    pub fn set_engine(&mut self, engine: SynthEngineType) -> bool {
        if engine == self.engine_type {
            return false;
        }

        self.engine = match engine {
            SynthEngineType::B3Organ => Box::new(Organ::new()),
            SynthEngineType::SubSynth => Box::new(synth::synth::Synth::new()),
            // SynthEngineType::SamplerSynth => {
            //     warn!("write SamplerSynth");
            //     return false;
            // }
            // SynthEngineType::WaveTableSynth => {
            //     warn!("write Wave Table synth");
            //     return false;
            // }
        };

        self.engine_type = engine;
        true
    }

    pub fn set_effect(&mut self, effect: EffectType) -> bool {
        // self.effect = EffectsModule::from(effect);

        self.effect.set(effect);

        self.effect_power = true;

        true
    }

    pub fn effect_toggle(&mut self) -> bool {
        self.effect_power = !self.effect_power;

        true
    }

    // fn apply_effect(effect: &mut impl Effect, sample: f32) -> f32 {
    //     effect.take_input(sample);
    //
    //     effect.get_sample()
    // }
}

impl SampleGen for Synth {
    fn get_sample(&mut self) -> f32 {
        if let Some(ref mut target) = self.lfo_target
            && self.lfo_routed
        {
            // info!("sending lfo data to target");
            let lfo_sample = self.lfo.get_sample();

            match target {
                LfoTarget::Synth(param) => self.engine.lfo_control(*param, lfo_sample),
                LfoTarget::Effect(param) => self.effect.lfo_control(*param, lfo_sample),
            }
        }

        let sample = self.engine.get_sample();

        if !self.effect_power {
            return sample;
        }

        // Self::get_sample
        self.effect.take_input(sample);

        self.effect.get_sample()
        // match self.effect {
        //     EffectsModule::Reverb(ref mut effect) => Self::apply_effect(effect, sample),
        //     EffectsModule::Chorus(ref mut effect) => {
        //         // warn!("chorus is not implemented yet!");
        //         // return sample;
        //         Self::apply_effect(effect, sample)
        //         // Self::apply_effect(effect, sample),
        //     } // EffectsModule::(effect) => self.apply_effect(effect, sample),
        // }
    }
}

// impl Synth {
//     pub fn get_state(&mut self) -> State {
//         let engine = self.engine_type;
//      let knob_params = self.engine.get_params();
//         let gui_params = self.engine.get_gui_params();
//         let effect = match self.effect {
//             EffectsModule::Reverb(_) => EffectType::Reverb,
//             EffectsModule::Chorus(_) => EffectType::Chorus,
//         };
//
//         State {
//             engine,
//             effect,
//             effect_on: self.effect_power,
//             knob_params,
//             gui_params,
//         }
//     }
// }
