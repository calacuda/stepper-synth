use crate::{
    effects::{Effect, EffectType, EffectsModule},
    pygame_coms::{GuiParam, Knob, SynthEngineType},
    HashMap, KnobCtrl, MidiControlled, SampleGen,
};
use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F32};
use enum_dispatch::enum_dispatch;
use log::*;
use midi_control::{Channel, MidiNote};
use midi_control::{ControlEvent, KeyEvent, MidiMessage};
use midi_out_engine::MidiOutEngine;
use organ::organ::Organ;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use std::{fmt::Debug, ops::IndexMut};
use strum::IntoEnumIterator;
// use wavetable_synth::MidiControlled as _;
// use synth_common::lfo::LFO;
use wave_table::WaveTableEngine;
use wurlitzer::WurlitzerEngine;

pub mod midi_out_engine;
pub mod organ;
pub mod synth;
pub mod synth_common;
pub mod wave_table;
pub mod wurlitzer;

#[enum_dispatch]
pub trait SynthEngine: Debug + SampleGen + KnobCtrl + Send + Clone {
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

#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "stepper_synth_backend", get_all, eq)
)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Param {
    Knob(Knob),
    Gui(GuiParam),
}

#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "stepper_synth_backend", get_all, eq)
)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum LfoTarget {
    Synth(Param),
    Effect(Param),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct LfoInput {
    pub target: Option<Param>,
    pub sample: f32,
}

#[enum_dispatch(SynthEngine)]
#[derive(Debug, Clone)]
pub enum SynthModule {
    B3Organ(Organ),
    SubSynth(synth::synth::Synth),
    Wurli(WurlitzerEngine),
    WaveTable(WaveTableEngine),
    MidiOutEngine(MidiOutEngine),
}

impl From<SynthEngineType> for SynthModule {
    fn from(value: SynthEngineType) -> Self {
        match value {
            SynthEngineType::B3Organ => Self::B3Organ(Organ::new()),
            SynthEngineType::SubSynth => Self::SubSynth(synth::synth::Synth::new()),
            SynthEngineType::Wurlitzer => Self::Wurli(WurlitzerEngine::new()),
            SynthEngineType::WaveTable => Self::WaveTable(WaveTableEngine::new()),
            SynthEngineType::MidiOut => Self::MidiOutEngine(MidiOutEngine::new()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SynthChannel {
    // pub lfo: LFO,
    pub engine: SynthModule,
    pub engine_type: SynthEngineType,
    // pub effect_power: bool,
    // pub effect: EffectsModules,
    pub effects: [Option<(EffectsModule, bool)>; 2],
    // pub effect_type: EffectType,
    // pub lfo_target: Option<LfoTarget>,
    // pub lfo_routed: bool,
    // pub stepper_state: StepperState,
    pub target_effects: bool,
}

impl From<SynthEngineType> for SynthChannel {
    fn from(value: SynthEngineType) -> Self {
        let engine = SynthModule::from(value);

        Self {
            engine,
            engine_type: value,
            effects: [None, None],
            target_effects: false,
        }
    }
}

impl SampleGen for SynthChannel {
    fn get_sample(&mut self) -> f32 {
        let sample = self.engine.get_sample();

        // if !self.effect_power || self.engine_type == SynthEngineType::WaveTable {
        //     return sample;
        // }
        //
        // self.get_effect().take_input(sample);
        //
        // self.get_effect().get_sample()

        let mut output = None;

        for effect in self.effects.iter_mut() {
            if let Some((ref mut effect, power)) = effect {
                if power.to_owned() {
                    match output {
                        Some(sample) => effect.take_input(sample),
                        None => effect.take_input(sample),
                    }

                    output = Some(effect.get_sample())
                }
            }
        }

        output.unwrap_or(sample)
    }
}

impl SynthChannel {
    pub fn set_engine(&mut self, engine: SynthEngineType) -> bool {
        if engine == self.engine_type {
            return false;
        }

        // if engine == SynthEngineType::WaveTable {
        //     self.effect_power = false;
        // }

        self.engine_type = engine;
        true
    }
}

#[derive(Debug, Clone)]
pub struct Synth {
    pub active_channel: usize,
    pub channels: Vec<SynthChannel>,
    all_pass: DirectForm1<f32>,
    // pub
}

impl SampleGen for Synth {
    fn get_sample(&mut self) -> f32 {
        let sum: f32 = self.channels.iter_mut().map(|chan| chan.get_sample()).sum();
        // sum / self.channels.len() as f32
        self.all_pass.run(sum)
    }
}

impl Synth {
    // pub fn new() -> Self {
    //     let engines = SynthEngineType::iter()
    //         .map(|engine_type| engine_type.into())
    //         .collect();
    //     let effects = EffectType::iter()
    //         .map(|effect_type| effect_type.into())
    //         .collect();
    //     let engine_type = SynthEngineType::B3Organ;
    //     // let engine_type = SynthEngineType::WaveTable;
    //
    //     Self {
    //         lfo: LFO::new(),
    //         // effect: EffectsModules::new(),
    //         effects,
    //         effect_type: EffectType::Reverb,
    //         effect_power: false,
    //         lfo_target: None,
    //         engine_type,
    //         engines,
    //         // engine: Box::new(Organ::new()),
    //         // engine: SynthEngines::new(),
    //         lfo_routed: false,
    //         // stepper_state: StepperState::default(),
    //         target_effects: false,
    //     }
    // }

    pub fn new() -> Self {
        info!("Synth::new() called!");
        println!("Synth::new() called!");

        // Cutoff and sampling frequencies
        let f0 = 10.hz();
        let fs = 1.khz();

        // Create coefficients for the biquads
        let coeffs =
            Coefficients::<f32>::from_params(Type::AllPass, fs, f0, Q_BUTTERWORTH_F32).unwrap();

        // Create two different biquads
        let all_pass = DirectForm1::<f32>::new(coeffs);

        Self {
            active_channel: 0,
            channels: [
                // SynthChannel::from(SynthEngineType::WaveTable),
                SynthChannel::from(SynthEngineType::B3Organ),
                SynthChannel::from(SynthEngineType::B3Organ),
                // SynthChannel::from(SynthEngineType::SubSynth),
                // SynthChannel::from(SynthEngineType::MidiOut),
            ]
            .to_vec(),
            all_pass,
        }
    }

    pub fn get_channel(&mut self) -> &mut SynthChannel {
        &mut self.channels[self.active_channel]
    }

    pub fn get_engine(&mut self) -> &mut SynthModule {
        // info!("{} => {}", self.engine_type, self.engine_type as usize);

        &mut self.get_channel().engine
        // .index_mut(self.engine_type as usize)
    }

    // pub fn get_effect(&mut self) -> &mut EffectsModule {
    //     // let effect = self.effect.;
    //     self.effects.index_mut(self.effect_type as usize)
    // }

    pub fn set_channel_engine(&mut self, channel: usize, engine: SynthEngineType) -> bool {
        if engine == self.channels[channel].engine_type {
            return false;
        }

        // if engine == SynthEngineType::WaveTable {
        //     self.effect_power = false;
        // }

        self.channels[channel].engine_type = engine;
        true
    }

    pub fn get_channel_engine(&mut self, channel: usize) -> &mut SynthChannel {
        let n_channels = self.channels.len();

        &mut self.channels[channel % n_channels]
    }

    // pub fn midi_on_channel(&mut self, message: &MidiMessage) {
    //     match *message {
    //         MidiMessage::Invalid => {
    //             error!("system received an invalid MIDI message.");
    //         }
    //         MidiMessage::NoteOn(channel, KeyEvent { key, value }) => {
    //             debug!("playing note: {key}");
    //             self.get_channel_engine(channel).engine.play(key, value)
    //         }
    //         MidiMessage::NoteOff(channel, KeyEvent { key, value: _ }) => {
    //             self.get_channel_engine(channel).engine.stop(key)
    //         }
    //         MidiMessage::PitchBend(channel, lsb, msb) => {
    //             let bend = i16::from_le_bytes([lsb, msb]) as f32 / (32_000.0 * 0.5) - 1.0;
    //
    //             if bend > 0.02 || bend < -0.020 {
    //                 self.get_channel_engine(channel).engine.bend(bend);
    //                 // send();
    //             } else {
    //                 self.get_channel_engine(channel).engine.unbend();
    //                 // send();
    //             }
    //         }
    //         MidiMessage::ControlChange(channel, ControlEvent { control, value }) => {
    //             let value = value as f32 / 127.0;
    //             // let effects = self.target_effects;
    //
    //             match &mut self.get_channel_engine(channel).engine {
    //                 SynthModule::WaveTable(wt) => {
    //                     wt.synth.midi_input(message);
    //                 }
    //                 engine => {
    //                     match control {
    //                         70 => engine.knob_1(value),
    //                         71 => engine.knob_2(value),
    //                         72 => engine.knob_3(value),
    //                         73 => engine.knob_4(value),
    //                         74 => engine.knob_5(value),
    //                         75 => engine.knob_6(value),
    //                         76 => engine.knob_7(value),
    //                         77 => engine.knob_8(value),
    //                         1 => engine.volume_swell(value),
    //                         _ => {
    //                             // info!("CC message => {control}-{value}");
    //                             false
    //                         }
    //                     };
    //                 }
    //             }
    //             // if self.engine_type == SynthEngineType::WaveTable {
    //             // } else {
    //             // }
    //         }
    //         _ => {}
    //     }
    // }

    // pub fn set_effect(&mut self, effect: EffectType) -> bool {
    //     // self.effect = EffectsModule::from(effect);
    //
    //     self.effect_type = effect;
    //
    //     // self.effect_power = true;
    //
    //     true
    // }

    // pub fn effect_toggle(&mut self) -> bool {
    //     self.effect_power = !self.effect_power;
    //
    //     // if self.engine_type == SynthEngineType::WaveTable {
    //     // self.effect_power = false;
    //     // }
    //
    //     true
    // }

    // pub fn route_lfo(&mut self, )
    // TODO: mod route
}

// impl SampleGen for Synth {
//     fn get_sample(&mut self) -> f32 {
//         // let engine = self.engines.index_mut(self.engine_type as usize);
//
//         if let Some(target) = self.lfo_target
//             && self.lfo_routed
//         {
//             // info!("sending lfo data to target");
//             let lfo_sample = self.lfo.get_sample();
//
//             match target {
//                 LfoTarget::Synth(_) => self.get_engine().lfo_control(lfo_sample),
//                 LfoTarget::Effect(_) => self.get_effect().lfo_control(lfo_sample),
//             }
//         }
//
//         // // let n_engines = self.engines.len();
//         // let mut n_samples = 1;
//         // // let mut sample = 0.0; // self.get_engine().get_sample() * 1.8;
//         //
//         // let mut samples: Vec<f32> = self
//         //     .engines
//         //     .iter_mut()
//         //     .map(|engine| {
//         //         let samp = engine.get_sample();
//         //
//         //         if samp != 0.0 {
//         //             // info!("{engine:?}");
//         //             n_samples += 1;
//         //         }
//         //
//         //         samp
//         //     })
//         //     .collect();
//         // samples[self.engine_type as usize] = samples[self.engine_type as usize] * 2.0;
//         //
//         // // let mut sample = samples.into_iter().sum();
//         //
//         // let bias = 1.0 / (n_samples as f32);
//         // let sample = samples.into_iter().sum::<f32>() * 0.8 * bias;
//         let sample = self.engines[self.engine_type as usize].get_sample();
//
//         if !self.effect_power || self.engine_type == SynthEngineType::WaveTable {
//             return sample;
//         }
//
//         self.get_effect().take_input(sample);
//
//         self.get_effect().get_sample()
//     }
// }

impl MidiControlled for Synth {
    fn midi_input(&mut self, message: &MidiMessage) {
        // if self.midi_sequencer.state.recording {
        //     self.midi_sequencer.midi_input(message);
        //     // return;
        // }

        match *message {
            MidiMessage::Invalid => {
                error!("system received an invalid MIDI message.");
            }
            MidiMessage::NoteOn(_, KeyEvent { key, value }) => {
                debug!("playing note: {key}");
                self.get_engine().play(key, value)
            }
            MidiMessage::NoteOff(_, KeyEvent { key, value: _ }) => self.get_engine().stop(key),
            MidiMessage::PitchBend(_, lsb, msb) => {
                let bend = i16::from_le_bytes([lsb, msb]) as f32 / (32_000.0 * 0.5) - 1.0;

                if bend > 0.02 || bend < -0.020 {
                    self.get_engine().bend(bend);
                    // send();
                } else {
                    self.get_engine().unbend();
                    // send();
                }
            }
            MidiMessage::ControlChange(_, ControlEvent { control, value }) => {
                let value = value as f32 / 127.0;
                // let effects = self.target_effects;

                match self.get_engine() {
                    SynthModule::WaveTable(wt) => {
                        wt.synth.midi_input(message);
                    }
                    engine => {
                        match control {
                            // 70 if effects => self.get_effect().knob_1(value),
                            // 71 if effects => self.get_effect().knob_2(value),
                            // 72 if effects => self.get_effect().knob_3(value),
                            // 73 if effects => self.get_effect().knob_4(value),
                            // 70 if !effects => self.get_engine().knob_1(value),
                            // 71 if !effects => self.get_engine().knob_2(value),
                            // 72 if !effects => self.get_engine().knob_3(value),
                            // 73 if !effects => self.get_engine().knob_4(value),
                            70 => engine.knob_1(value),
                            71 => engine.knob_2(value),
                            72 => engine.knob_3(value),
                            73 => engine.knob_4(value),
                            74 => engine.knob_5(value),
                            75 => engine.knob_6(value),
                            76 => engine.knob_7(value),
                            77 => engine.knob_8(value),
                            1 => engine.volume_swell(value),
                            _ => {
                                // info!("CC message => {control}-{value}");
                                false
                            }
                        };
                    }
                }
                // if self.engine_type == SynthEngineType::WaveTable {
                // } else {
                // }
            }
            _ => {}
        }
    }
}
