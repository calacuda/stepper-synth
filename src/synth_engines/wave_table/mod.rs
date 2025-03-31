use super::{LfoInput, SynthEngine};
use crate::{
    pygame_coms::{GuiParam, Knob},
    HashMap, KnobCtrl, SampleGen,
};
use log::warn;
use midi_control::MidiNote;
use wavetable_synth::{
    common::{ModMatrixDest, ModMatrixItem, ModMatrixSrc, OscParam},
    App,
};

pub mod wavetable_synth;

#[derive(Debug, Clone)]
pub struct WaveTableEngine {
    pub synth: App,
    lfo_target: LfoInput,
}

impl WaveTableEngine {
    pub fn new() -> Self {
        let mut synth = App::default();

        synth
            .voices
            .iter_mut()
            .for_each(|voice| voice.oscs.iter_mut().for_each(|(osc, _on)| osc.level = 0.5));
        synth.mod_matrix[0] = Some(ModMatrixItem {
            src: ModMatrixSrc::Velocity,
            dest: ModMatrixDest::SynthVolume,
            amt: 1.0,
            bipolar: false,
        });
        synth.mod_matrix[1] = Some(ModMatrixItem {
            src: ModMatrixSrc::Velocity,
            dest: ModMatrixDest::Osc {
                osc: 0,
                param: OscParam::Level,
            },
            amt: 1.0,
            bipolar: false,
        });
        synth.mod_matrix[2] = Some(ModMatrixItem {
            src: ModMatrixSrc::Velocity,
            dest: ModMatrixDest::Osc {
                osc: 1,
                param: OscParam::Level,
            },
            amt: 1.0,
            bipolar: false,
        });
        synth.mod_matrix[3] = Some(ModMatrixItem {
            src: ModMatrixSrc::Velocity,
            dest: ModMatrixDest::Osc {
                osc: 2,
                param: OscParam::Level,
            },
            amt: 1.0,
            bipolar: false,
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
