use crate::{
    pygame_coms::{GuiParam, Knob},
    HashMap, KnobCtrl, SampleGen,
};
use log::warn;
use midi_control::MidiNote;
// #[cfg(feature = "pyo3")]
// use pyo3::prelude::*;
use wavetable_synth::{App, SampleGen as _};

use super::{LfoInput, SynthEngine};

#[derive(Debug, Clone)]
pub struct WaveTableEngine {
    pub synth: App,
    lfo_target: LfoInput,
}

impl WaveTableEngine {
    pub fn new() -> Self {
        let synth = App::default();

        synth.voices.iter().for_each(|voice| {
            voice
                .lock()
                .unwrap()
                .oscs
                .iter_mut()
                .for_each(|(osc, _on)| osc.level = 0.8)
        });

        Self {
            synth,
            lfo_target: LfoInput::default(),
        }
    }
}

impl SampleGen for WaveTableEngine {
    fn get_sample(&mut self) -> f32 {
        self.synth.get_sample()
    }
}

impl KnobCtrl for WaveTableEngine {
    fn get_lfo_input(&mut self) -> &mut LfoInput {
        &mut self.lfo_target
    }

    // TODO: impl the knobs control functions and make them into midi messages
}

impl SynthEngine for WaveTableEngine {
    fn name(&self) -> String {
        "WaveTable".into()
    }

    fn play(&mut self, note: MidiNote, velocity: u8) {
        self.synth.play(note, velocity)
    }

    fn stop(&mut self, note: MidiNote) {
        self.synth.stop(note);
    }

    fn bend(&mut self, _amount: f32) {
        warn!("bending not yet implemented for WaveTableEngine")
    }

    fn unbend(&mut self) {
        warn!("bending not yet implemented for WaveTableEngine")
    }

    fn volume_swell(&mut self, _amount: f32) -> bool {
        false
    }

    fn get_params(&self) -> HashMap<Knob, f32> {
        let map = HashMap::default();
        map
    }

    fn get_gui_params(&self) -> HashMap<GuiParam, f32> {
        let map = HashMap::default();

        map
    }
}
