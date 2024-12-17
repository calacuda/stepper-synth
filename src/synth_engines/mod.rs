use crate::{
    effects::Effect,
    pygame_coms::{GuiParam, Knob, State, SynthEngineType},
    KnobCtrl, SampleGen,
};
use log::info;
use midi_control::MidiNote;
use organ::organ::Organ;
use std::{collections::HashMap, fmt::Debug};
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
    fn get_params(&mut self) -> HashMap<Knob, f32>;
    fn get_gui_params(&mut self) -> HashMap<GuiParam, f32>;
}

#[derive(Debug, Clone)]
pub enum LfoTarget {}

#[derive(Debug)]
pub struct Synth {
    pub lfo: LFO,
    pub engine: Box<dyn SynthEngine>,
    pub engine_type: SynthEngineType,
    pub effect: Option<Box<dyn Effect>>,
    // pub effect_power: bool,
    pub lfo_target: Option<LfoTarget>,
}

impl Synth {
    pub fn new() -> Self {
        Self {
            lfo: LFO::new(),
            effect: None,
            lfo_target: None,
            engine_type: SynthEngineType::B3Organ,
            engine: Box::new(Organ::new()),
        }
    }

    pub fn set_engine(&mut self, engine: SynthEngineType) -> bool {
        if engine == self.engine_type {
            return false;
        }

        self.engine = match engine {
            SynthEngineType::B3Organ => Box::new(Organ::new()),
            SynthEngineType::SubSynth => Box::new(synth::synth::Synth::new()),
            SynthEngineType::SamplerSynth => todo!("write SamplerSynth"),
        };

        self.engine_type = engine;
        true
    }
}

impl SampleGen for Synth {
    fn get_sample(&mut self) -> f32 {
        if let Some(ref mut _target) = self.lfo_target {
            info!("sending lfo data to target");
        }

        let sample = self.engine.get_sample();

        let Some(ref mut effect) = self.effect else {
            return sample;
        };

        effect.take_input(sample);

        effect.get_sample()
    }
}

impl Synth {
    pub fn get_state(&mut self) -> State {
        let engine = self.engine_type;
        let knob_params = self.engine.get_params();
        let gui_params = self.engine.get_gui_params();

        State {
            engine,
            knob_params,
            gui_params,
        }
    }
}
