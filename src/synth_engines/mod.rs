use crate::{
    effects::{Effect, EffectType, EffectsModule},
    pygame_coms::{GuiParam, Knob, SynthEngineType},
    sequencer::{SequencerIntake, StepperState},
    HashMap, KnobCtrl, MidiControlled, SampleGen,
};
use enum_dispatch::enum_dispatch;
use log::*;
use midi_control::MidiNote;
use midi_control::{ControlEvent, KeyEvent, MidiMessage};
use organ::organ::Organ;
use pyo3::prelude::*;
use std::{fmt::Debug, ops::IndexMut};
use strum::IntoEnumIterator;
use synth_common::lfo::LFO;

pub mod organ;
pub mod synth;
pub mod synth_common;

#[enum_dispatch]
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
    // TODO: impl sustain_peddal
    // fn sustain_peddal(&mut self);
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

#[enum_dispatch(SynthEngine)]
#[derive(Debug)]
pub enum SynthModule {
    B3Organ(Organ),
    SubSynth(synth::synth::Synth),
}

impl From<SynthEngineType> for SynthModule {
    fn from(value: SynthEngineType) -> Self {
        match value {
            SynthEngineType::B3Organ => Self::B3Organ(Organ::new()),
            SynthEngineType::SubSynth => Self::SubSynth(synth::synth::Synth::new()),
        }
    }
}

#[derive(Debug)]
pub struct Synth {
    pub lfo: LFO,
    pub engines: Box<[SynthModule]>,
    pub engine_type: SynthEngineType,
    pub effect_power: bool,
    // pub effect: EffectsModules,
    pub effects: Box<[EffectsModule]>,
    pub effect_type: EffectType,
    // pub effect_power: bool,
    pub lfo_target: Option<LfoTarget>,
    pub lfo_routed: bool,
    pub midi_sequencer: SequencerIntake,
    // pub stepper_state: StepperState,
    pub target_effects: bool,
}

impl Synth {
    pub fn new() -> Self {
        let engines = SynthEngineType::iter()
            .map(|engine_type| engine_type.into())
            .collect();
        let effects = EffectType::iter()
            .map(|effect_type| effect_type.into())
            .collect();

        Self {
            lfo: LFO::new(),
            // effect: EffectsModules::new(),
            effects,
            effect_type: EffectType::Reverb,
            effect_power: false,
            lfo_target: None,
            engine_type: SynthEngineType::B3Organ,
            engines,
            // engine: Box::new(Organ::new()),
            // engine: SynthEngines::new(),
            lfo_routed: false,
            midi_sequencer: SequencerIntake::new(),
            // stepper_state: StepperState::default(),
            target_effects: false,
        }
    }

    pub fn get_engine(&mut self) -> &mut SynthModule {
        self.engines.index_mut(self.engine_type as usize)
    }

    pub fn get_effect(&mut self) -> &mut EffectsModule {
        // let effect = self.effect.;
        self.effects.index_mut(self.effect_type as usize)
    }

    pub fn set_engine(&mut self, engine: SynthEngineType) -> bool {
        if engine == self.engine_type {
            return false;
        }

        self.engine_type = engine;
        true
    }

    pub fn set_effect(&mut self, effect: EffectType) -> bool {
        // self.effect = EffectsModule::from(effect);

        self.effect_type = effect;

        // self.effect_power = true;

        true
    }

    pub fn effect_toggle(&mut self) -> bool {
        self.effect_power = !self.effect_power;

        true
    }
}

impl SampleGen for Synth {
    fn get_sample(&mut self) -> f32 {
        // let engine = self.engines.index_mut(self.engine_type as usize);

        if let Some(target) = self.lfo_target
            && self.lfo_routed
        {
            // info!("sending lfo data to target");
            let lfo_sample = self.lfo.get_sample();

            match target {
                LfoTarget::Synth(param) => self.get_engine().lfo_control(param, lfo_sample),
                LfoTarget::Effect(param) => self.get_effect().lfo_control(param, lfo_sample),
            }
        }

        let sample = self.get_engine().get_sample();

        if !self.effect_power {
            return sample;
        }

        self.get_effect().take_input(sample);

        self.get_effect().get_sample()
    }
}

impl MidiControlled for Synth {
    fn midi_input(&mut self, message: &MidiMessage) {
        if self.midi_sequencer.state.recording {
            self.midi_sequencer.midi_input(message);
            // return;
        }

        match *message {
            MidiMessage::Invalid => {
                error!("system recieved an invalid MIDI message.");
            }
            MidiMessage::NoteOn(_, KeyEvent { key, value }) => {
                debug!("playing note: {key}");
                self.get_engine().play(key, value)
            }
            MidiMessage::NoteOff(_, KeyEvent { key, value: _ }) => self.get_engine().stop(key),
            MidiMessage::PitchBend(_, lsb, msb) => {
                let bend = i16::from_le_bytes([lsb, msb]) as f32 / (32_000.0 * 0.5) - 1.0;

                if bend > 0.026 || bend < -0.026 {
                    self.get_engine().bend(bend);
                    // send();
                } else {
                    self.get_engine().unbend();
                    // send();
                }
            }
            MidiMessage::ControlChange(_, ControlEvent { control, value }) => {
                let value = value as f32 / 127.0;

                match control {
                    70 if self.target_effects => self.get_effect().knob_1(value),
                    71 if self.target_effects => self.get_effect().knob_2(value),
                    72 if self.target_effects => self.get_effect().knob_3(value),
                    73 if self.target_effects => self.get_effect().knob_4(value),
                    70 if !self.target_effects => self.get_engine().knob_1(value),
                    71 if !self.target_effects => self.get_engine().knob_2(value),
                    72 if !self.target_effects => self.get_engine().knob_3(value),
                    73 if !self.target_effects => self.get_engine().knob_4(value),
                    74 => self.get_engine().knob_5(value),
                    75 => self.get_engine().knob_6(value),
                    76 => self.get_engine().knob_7(value),
                    77 => self.get_engine().knob_8(value),
                    1 => self.get_engine().volume_swell(value),
                    117 => {
                        self.midi_sequencer.state.playing = false;
                        self.midi_sequencer.state.recording = false;
                        false
                    }
                    118 => {
                        self.midi_sequencer.state.playing = true;
                        self.midi_sequencer.state.recording = false;
                        false
                    }
                    119 => {
                        self.midi_sequencer.state.playing = false;
                        self.midi_sequencer.state.recording = true;
                        false
                    }
                    _ => {
                        info!("CC message => {control}-{value}");
                        false
                    }
                };
            }
            _ => {}
        }
    }
}
