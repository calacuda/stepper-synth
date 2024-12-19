use crate::{
    pygame_coms::{GuiParam, Knob},
    synth_engines::{
        synth_common::env::{ATTACK, DECAY, RELEASE, SUSTAIN},
        SynthEngine,
    },
    KnobCtrl, SampleGen,
};
use midi_control::MidiNote;
use std::collections::HashMap;

use super::{osc::SynthOscillator, OscType};

pub const VOICES: usize = 10;

#[derive(Debug)]
pub struct Synth {
    pub osc_s: [(Vec<SynthOscillator>, i16); 2],
    pub osc_type: [(OscType, f32); 2],
    // pub overtones: [Overtone; 10],
    pub volume: f32,
    pub mix: f32,
}

impl Synth {
    pub fn new() -> Self {
        let osc_1: Vec<SynthOscillator> = (0..VOICES).map(|_| SynthOscillator::new()).collect();
        let osc_2: Vec<SynthOscillator> = (0..VOICES).map(|_| SynthOscillator::new()).collect();

        Self {
            osc_s: [(osc_1, 0), (osc_2, 0)],
            // wave_tables,
            osc_type: [
                // (OscType::Sin, 1.0),
                (OscType::Saw, 1.0),
                (OscType::Saw, 1.0),
                // (OscType::Saw, 1.0),
                // (OscType::Tri, 0.75),
                // (OscType::Sqr, 1.0),
            ],
            // overtones,
            // osc_type: Arc::new([(OscType::Tri, 1.0)]),
            volume: 1.0,
            mix: 0.5,
        }
    }

    pub fn get_sample(&mut self) -> f32 {
        let mut sample = 0.0;
        // println!("lfo sample {lfo_sample}");

        for ((osc_s, _offset), mix) in self
            .osc_s
            .iter_mut()
            // .zip(self.wave_tables.index(&self.osc_type.clone().into()).iter())
            .zip([1.0 - self.mix, self.mix])
        {
            // println!("{:?}", osc_s.len());
            // info!("mix :  {mix}");
            for osc in osc_s {
                if osc.playing.is_some() {
                    // osc.for_each(|(osc, _offset)| {
                    // info!("mix :  {mix}");

                    // println!("playing");
                    sample += osc.get_sample() * mix;
                    // sample += osc.get_sample(&wave_table) * volume * self.mix;
                    // println!(
                    //     "env => {}, {}",
                    //     osc.env_filter.get_samnple(),
                    //     osc.env_filter.phase
                    // );
                    // });
                }
            }
        }

        sample *= self.volume;
        sample.tanh()
        // println!("synth sample => {sample}");
        // sample * self.volume
    }

    pub fn play(&mut self, midi_note: MidiNote, _velocity: u8) {
        // let midi_note = if midi_note >= 12 {
        //     midi_note - 12
        // } else {
        //     return;
        // };

        for (osc_s, _offset) in self.osc_s.iter_mut() {
            for osc in osc_s {
                if osc.playing == Some(midi_note) && osc.env_filter.phase != RELEASE {
                    return;
                }
            }
        }

        for (osc_s, offset) in self.osc_s.iter_mut() {
            for osc in osc_s {
                if osc.playing.is_none() {
                    // let note = if *offset > 0 {
                    //     midi_note + (*offset as u8)
                    // } else {
                    //     // println!("offset {} -> {}", offset, (offset.abs() as u8));
                    //     midi_note - (offset.abs() as u8)
                    // };
                    let note = if midi_note >= (*offset as u8) {
                        midi_note - (*offset as u8)
                    } else {
                        // println!("offset {} -> {}", offset, (offset.abs() as u8));
                        midi_note // - (offset.abs() as u8)
                    };

                    osc.press(note);
                    osc.playing = Some(midi_note);
                    // println!("playing note on osc {i}");

                    break;
                }
            }
        }
    }

    pub fn stop(&mut self, midi_note: MidiNote) {
        // let midi_note = if midi_note >= 12 {
        //     midi_note - 12
        // } else {
        //     return;
        // };

        for (osc_s, _offset) in self.osc_s.iter_mut() {
            for osc in osc_s {
                // let note = if *offset > 0 {
                //     midi_note + (*offset as u8)
                // } else {
                //     // println!("offset {} -> {}", offset, (offset.abs() as u8));
                //     midi_note - (offset.abs() as u8)
                // };

                if osc.playing == Some(midi_note) && osc.env_filter.phase != RELEASE {
                    // println!("release");
                    osc.release();
                    break;
                }
            }
        }
    }

    pub fn bend_all(&mut self, bend: f32) {
        for (osc_s, _offset) in self.osc_s.iter_mut() {
            for osc in osc_s {
                if osc.playing.is_some() {
                    osc.bend(bend);
                }
            }
        }
    }

    pub fn unbend(&mut self) {
        for (osc_s, _offset) in self.osc_s.iter_mut() {
            for osc in osc_s {
                if osc.playing.is_some() {
                    osc.unbend();
                }
            }
        }
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol;
    }

    pub fn set_atk(&mut self, atk: f32) {
        for (osc_s, _offset) in self.osc_s.iter_mut() {
            for osc in osc_s {
                osc.env_filter.set_atk(atk);
            }
        }
    }

    pub fn set_decay(&mut self, decay: f32) {
        for (osc_s, _offset) in self.osc_s.iter_mut() {
            for osc in osc_s {
                osc.env_filter.set_decay(decay);
            }
        }
    }

    pub fn set_sus(&mut self, sus: f32) {
        for (osc_s, _offset) in self.osc_s.iter_mut() {
            for osc in osc_s {
                osc.env_filter.set_sus(sus);
            }
        }
    }

    pub fn set_release(&mut self, release: f32) {
        for (osc_s, _offset) in self.osc_s.iter_mut() {
            for osc in osc_s {
                osc.env_filter.set_release(release);
            }
        }
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        // let cutoff = cutoff * 16_000.0;

        for (osc_s, _offset) in self.osc_s.iter_mut() {
            for osc in osc_s {
                osc.low_pass.set_cutoff(cutoff);
            }
        }
    }

    pub fn set_resonace(&mut self, resonace: f32) {
        for (osc_s, _offset) in self.osc_s.iter_mut() {
            for osc in osc_s {
                osc.low_pass.set_resonace(resonace);
            }
        }
    }
}

impl SampleGen for Synth {
    fn get_sample(&mut self) -> f32 {
        self.get_sample()
    }
}

impl SynthEngine for Synth {
    fn name(&self) -> String {
        "Synth".into()
    }

    fn play(&mut self, note: MidiNote, velocity: u8) {
        self.play(note, velocity)
    }

    fn stop(&mut self, note: MidiNote) {
        self.stop(note)
    }

    fn bend(&mut self, amount: f32) {
        self.bend_all(amount)
    }

    fn get_params(&mut self) -> HashMap<Knob, f32> {
        let mut map = HashMap::with_capacity(8);

        map.insert(Knob::One, self.osc_s[0].0[0].env_filter.base_params[ATTACK]);
        map.insert(Knob::Two, self.osc_s[0].0[0].env_filter.base_params[DECAY]);
        map.insert(
            Knob::Three,
            self.osc_s[0].0[0].env_filter.base_params[SUSTAIN],
        );
        map.insert(
            Knob::Four,
            self.osc_s[0].0[0].env_filter.base_params[RELEASE],
        );
        map.insert(Knob::Five, self.osc_s[0].0[0].low_pass.cutoff);
        map.insert(Knob::Six, self.osc_s[0].0[0].low_pass.resonance);
        // map.insert(Knob::Seven, self.overtones[6].volume as f32);
        // map.insert(Knob::Eight, self.overtones[7].volume as f32);

        map
    }

    fn get_gui_params(&mut self) -> HashMap<GuiParam, f32> {
        let mut map = HashMap::with_capacity(8);

        // osc_1 type
        map.insert(GuiParam::A, self.osc_type[0].0 as usize as f32);
        // osc_2 type
        map.insert(GuiParam::B, self.osc_type[1].0 as usize as f32);
        // mix
        map.insert(GuiParam::C, self.mix);
        // osc_2 note offset
        map.insert(GuiParam::D, self.osc_s[1].1 as f32);
        // detune
        map.insert(GuiParam::E, 0.0);

        map
    }

    fn volume_swell(&mut self, amount: f32) -> bool {
        self.volume = amount;
        false
    }
}

impl KnobCtrl for Synth {
    fn knob_1(&mut self, value: f32) -> bool {
        self.set_atk(value);
        true
    }

    fn knob_2(&mut self, value: f32) -> bool {
        self.set_decay(value);
        true
    }

    fn knob_3(&mut self, value: f32) -> bool {
        self.set_sus(value);
        true
    }

    fn knob_4(&mut self, value: f32) -> bool {
        self.set_release(value);
        true
    }

    fn knob_5(&mut self, value: f32) -> bool {
        self.set_cutoff(value);
        true
    }

    fn knob_6(&mut self, value: f32) -> bool {
        self.set_resonace(value);
        true
    }

    fn gui_param_1(&mut self, value: f32) -> bool {
        self.osc_type[0].0 = OscType::from(value as usize);
        for osc in self.osc_s[0].0.iter_mut() {
            osc.set_osc_type(self.osc_type[0].0)
        }

        true
    }

    fn gui_param_2(&mut self, value: f32) -> bool {
        self.osc_type[1].0 = OscType::from(value as usize);

        for osc in self.osc_s[1].0.iter_mut() {
            osc.set_osc_type(self.osc_type[1].0)
        }

        true
    }

    fn gui_param_3(&mut self, value: f32) -> bool {
        self.mix = value;
        true
    }

    fn gui_param_4(&mut self, value: f32) -> bool {
        // info!("value = {value}");
        self.osc_s[1].1 = value as i16;
        true
    }
}
