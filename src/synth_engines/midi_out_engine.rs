use super::{LfoInput, SynthEngine};
use crate::{
    pygame_coms::{GuiParam, Knob},
    HashMap, KnobCtrl, SampleGen,
};
use midi_control::MidiNote;

#[derive(Debug, Clone)]
pub struct MidiOutEngine {
    /// the output device used to send the midi data
    pub output_dev: Option<String>,
    /// the channel to send the midi data on.
    pub output_channel: u8,
    lfo_target: LfoInput,
}

impl MidiOutEngine {
    pub fn new() -> Self {
        Self {
            output_dev: None,
            output_channel: 0,
            lfo_target: LfoInput::default(),
        }
    }
}

impl SampleGen for MidiOutEngine {
    fn get_sample(&mut self) -> f32 {
        0.0
    }
}

impl KnobCtrl for MidiOutEngine {
    fn get_lfo_input(&mut self) -> &mut LfoInput {
        &mut self.lfo_target
    }

    // TODO: impl the knobs control functions and make them into midi messages
}

impl SynthEngine for MidiOutEngine {
    fn name(&self) -> String {
        format!(
            "Midi Out {}",
            self.output_dev.clone().unwrap_or("ANY".into())
        )
    }

    fn play(&mut self, _note: MidiNote, _velocity: u8) {
        // TODO: send play note command to output device
    }

    fn stop(&mut self, _note: MidiNote) {
        // TODO: send stop note command to output device
    }

    fn bend(&mut self, _amount: f32) {
        // TODO: send pitch bend command to output device
    }

    fn unbend(&mut self) {
        // TODO: send pitch bend is zero command to output device
    }

    fn get_params(&self) -> HashMap<Knob, f32> {
        HashMap::default()
    }

    fn get_gui_params(&self) -> HashMap<GuiParam, f32> {
        // let mut params = HashMap::default();
        // params.insert(GuiParam::A, );
        // params
        HashMap::default()
    }

    fn volume_swell(&mut self, _amount: f32) -> bool {
        false
    }
}
