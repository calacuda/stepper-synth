use crate::{effects::Effect, KnobCtrl, SampleGen};
use log::info;
use midi_control::MidiNote;
use organ::organ::Organ;
use synth_common::lfo::LFO;

pub mod organ;
// pub mod synth;
pub mod synth_common;

pub trait SynthEngine: SampleGen + KnobCtrl + Send {
    fn name(&self) -> String;

    fn play(&mut self, note: MidiNote, velocity: u8);
    fn stop(&mut self, note: MidiNote);
    fn bend(&mut self, amount: f32);
    fn unbend(&mut self) {
        self.bend(0.0);
    }
    fn volume_swell(&mut self, amount: f32);
}

pub enum LfoTarget {}

pub struct Synth {
    pub lfo: LFO,
    pub engine: Box<dyn SynthEngine>,
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
            engine: Box::new(Organ::new()),
        }
    }
}

// impl SynthEngine for Synth {
//     fn name(&self) -> String {
//         String::new()
//     }
//
//     fn play(&mut self, note: MidiNote) {
//         self.engine.play(note)
//     }
//
//     fn stop(&mut self, note: MidiNote) {
//         self.engine.stop(note)
//     }
//
//     fn bend(&mut self, amount: f32) {
//         self.engine.bend(amount)
//     }
// }

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
