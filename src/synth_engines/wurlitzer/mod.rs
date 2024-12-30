use super::{synth_common::env::RELEASE, LfoInput, SynthEngine};
use crate::{
    pygame_coms::{GuiParam, Knob},
    HashMap, KnobCtrl, SampleGen,
};
use midi_control::MidiNote;
use note_osc::WurliNoteOsc;

pub mod note_osc;

#[derive(Debug, Clone)]
pub struct WurlitzerEngine {
    pub osc_s: Vec<WurliNoteOsc>,
    lfo_target: LfoInput,
}

impl WurlitzerEngine {
    pub fn new() -> Self {
        let osc_s = (0..10).map(|_| WurliNoteOsc::new()).collect();
        let lfo_target = LfoInput::default();

        Self { osc_s, lfo_target }
    }

    pub fn play(&mut self, midi_note: MidiNote, velocity: u8) {
        // info!("wurlitzer play note {midi_note}");

        for osc in self.osc_s.iter_mut() {
            if osc.playing == Some(midi_note) && osc.vol_env.pressed() {
                return;
            }
        }

        for osc in self.osc_s.iter_mut() {
            if osc.playing.is_none() && !osc.vol_env.pressed() {
                let vel = velocity as f32 / (u8::MAX as f32 * 0.5);
                osc.press(midi_note, vel);
                osc.playing = Some(midi_note);

                break;
            }
        }
    }

    pub fn stop(&mut self, midi_note: MidiNote) {
        for osc in self.osc_s.iter_mut() {
            if osc.playing == Some(midi_note) && osc.vol_env.phase != RELEASE {
                osc.release();
                break;
            }
        }
    }

    fn set_trem_depth(&mut self, depth: f32) {
        self.osc_s
            .iter_mut()
            .for_each(|osc| osc.set_trem_depth(depth))
    }

    pub fn bend_all(&mut self, bend: f32) {
        for osc in self.osc_s.iter_mut() {
            if osc.playing.is_some() {
                osc.bend(bend);
            }
        }
    }

    pub fn unbend(&mut self) {
        for osc in self.osc_s.iter_mut() {
            if osc.playing.is_some() {
                osc.unbend();
            }
        }
    }
}

impl SampleGen for WurlitzerEngine {
    fn get_sample(&mut self) -> f32 {
        // let mut n_samples = 1;
        // let mut sample = 0.0;

        let samples = self.osc_s.iter_mut().map(|osc| osc.get_sample());
        let sample: f32 = samples.sum();
        // for osc in self.osc_s.iter_mut() {
        //     if osc.playing.is_some() {
        //         // n_samples += 1;
        //         sample += osc.get_sample() * 0.75;
        //     }
        // }

        // sample * 0.75
        (sample * 0.5).tanh()
    }
}

impl KnobCtrl for WurlitzerEngine {
    fn get_lfo_input(&mut self) -> &mut super::LfoInput {
        &mut self.lfo_target
    }

    fn knob_1(&mut self, value: f32) -> bool {
        self.set_trem_depth(value);
        false
    }
}

impl SynthEngine for WurlitzerEngine {
    fn name(&self) -> String {
        "Wurlitzer".into()
    }

    fn play(&mut self, note: MidiNote, velocity: u8) {
        self.play(note, velocity);
    }

    fn stop(&mut self, note: MidiNote) {
        self.stop(note);
    }

    fn bend(&mut self, amount: f32) {
        self.bend_all(amount);
    }

    fn unbend(&mut self) {
        self.unbend();
    }

    fn volume_swell(&mut self, _amount: f32) -> bool {
        false
    }

    fn get_params(&self) -> HashMap<Knob, f32> {
        let mut map = HashMap::default();

        map.insert(Knob::One, self.osc_s[0].trem_lfo_depth);
        // map.insert(Knob::Two, self.osc_s[0].0[0].env_filter.base_params[DECAY]);
        // map.insert(
        //     Knob::Three,
        //     self.osc_s[0].0[0].env_filter.base_params[SUSTAIN],
        // );
        // map.insert(
        //     Knob::Four,
        //     self.osc_s[0].0[0].env_filter.base_params[RELEASE],
        // );
        // map.insert(Knob::Five, self.osc_s[0].0[0].low_pass.cutoff);
        // map.insert(Knob::Six, self.osc_s[0].0[0].low_pass.resonance);
        // map.insert(Knob::Seven, self.overtones[6].volume as f32);
        // map.insert(Knob::Eight, self.overtones[7].volume as f32);

        map
    }

    fn get_gui_params(&self) -> HashMap<GuiParam, f32> {
        let map = HashMap::default();

        // // osc_1 type
        // map.insert(GuiParam::A, self.osc_type[0].0 as usize as f32);
        // // osc_2 type
        // map.insert(GuiParam::B, self.osc_type[1].0 as usize as f32);
        // // mix
        // map.insert(GuiParam::C, self.mix);
        // // osc_2 note offset
        // map.insert(GuiParam::D, self.osc_s[1].1 as f32);
        // // detune
        // map.insert(GuiParam::E, 0.0);

        map
    }
}
