use crate::{
    pygame_coms::{GuiParam, Knob},
    synth_engines::{
        synth_common::{
            env::{ATTACK, DECAY, RELEASE, SUSTAIN},
            lfo::LFO,
            osc::{Oscillator, Overtone},
        },
        SynthEngine,
    },
    KnobCtrl, SampleGen,
};
use midi_control::MidiNote;
use std::{collections::HashMap, sync::Arc};

pub type WaveTable = Arc<[f32]>;
// pub type WaveTables = [(WaveTable, f32); 2];

pub const WAVE_TABLE_SIZE: usize = 126;
pub const VOICES: usize = 10;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum OscType {
    Sin,
    Tri,
    Sqr,
    Saw,
}

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct WaveTables {
    pub sin: WaveTable,
    pub tri: WaveTable,
    pub sqr: WaveTable,
    pub saw: WaveTable,
}

impl WaveTables {
    pub fn new(overtones: &[Overtone]) -> Self {
        Self {
            sin: Self::build_sine_table(overtones),
            tri: Self::build_triangle_table(overtones),
            sqr: Self::build_square_table(overtones),
            saw: Self::build_saw_table(overtones),
        }
    }

    fn build_saw_table(overtones: &[Overtone]) -> WaveTable {
        let mut wave_table = [0.0; WAVE_TABLE_SIZE];

        let n_overtones = overtones.len();

        let bias = 1.0 / n_overtones as f32;

        for i in 0..WAVE_TABLE_SIZE {
            for ot in overtones {
                // wave_table[i] += (((i as f64 % ot.overtone) - 1.0) * ot.volume) as f32
                wave_table[i] +=
                    ((((i as f64 * ((4.0 * ot.overtone) / WAVE_TABLE_SIZE as f64)) % 2.0) - 1.0)
                        * ot.volume) as f32;
                // break;
            }

            wave_table[i] *= bias;
            // println!("saw tooth => {}", wave_table[i]);
        }

        wave_table.into()
    }

    fn build_square_table(overtones: &[Overtone]) -> WaveTable {
        let mut wave_table = [0.0; WAVE_TABLE_SIZE];

        let n_overtones = overtones.len();

        let bias = 1.0 / n_overtones as f32;

        for i in 0..WAVE_TABLE_SIZE {
            for ot in overtones {
                if (i as f64 % ot.overtone as f64) < 1.0 {
                    wave_table[i] += ot.volume as f32
                }
            }

            wave_table[i] *= bias;
        }

        wave_table.into()
    }

    fn build_triangle_table(overtones: &[Overtone]) -> WaveTable {
        let mut wave_table = [0.0; WAVE_TABLE_SIZE];

        let n_overtones = overtones.len();

        let bias = 1.0 / n_overtones as f32;

        for i in 0..WAVE_TABLE_SIZE {
            for ot in overtones {
                wave_table[i] += (((i as f64 % ot.overtone as f64) - 1.0).abs() * ot.volume) as f32
            }

            wave_table[i] *= bias;
        }

        // println!("bigest build_triangle_table {:?}", wave_table.iter().max());

        wave_table.into()
    }

    fn build_sine_table(overtones: &[Overtone]) -> WaveTable {
        let mut wave_table = [0.0; WAVE_TABLE_SIZE];

        let n_overtones = overtones
            .iter()
            .filter(|tone| tone.volume > 0.0)
            .collect::<Vec<_>>()
            .len();

        let bias = 1.0 / n_overtones as f32;

        for i in 0..WAVE_TABLE_SIZE {
            for ot in overtones {
                wave_table[i] += ((2.0 * core::f64::consts::PI * i as f64 * ot.overtone
                    / WAVE_TABLE_SIZE as f64)
                    .sin()
                    * ot.volume) as f32
            }

            wave_table[i] *= bias;
        }

        wave_table.into()
    }
}

#[derive(Debug, Clone)]
pub struct Organ {
    pub osc_s: [Oscillator; VOICES],
    pub wave_table: WaveTable,
    pub osc_type: OscType,
    pub overtones: [Overtone; 8],
    pub lfo: LFO,
    pub volume: f32,
    // pub chorus: Chorus,
    // pub reverb: Reverb,
}

impl Organ {
    pub fn new() -> Self {
        let overtones = [
            Overtone {
                overtone: 0.5_f64.powf(1.0 / 12.0),
                volume: 1.0,
            },
            Overtone {
                // overtone: 2.0_f64.powf(1.0 / 12.0),
                overtone: 1.5_f64.powf(1.0 / 12.0),
                volume: 1.0,
            },
            Overtone {
                overtone: 1.0,
                volume: 1.0,
            },
            Overtone {
                overtone: 3.0,
                // overtone: 4.0,
                volume: 0.5,
            },
            Overtone {
                overtone: 4.0,
                // overtone: 8.0,
                volume: 0.0,
            },
            Overtone {
                overtone: 5.0,
                // overtone: 16.0,
                volume: 0.0,
            },
            Overtone {
                overtone: 6.0,
                // overtone: 32.0,
                volume: 0.0,
            },
            Overtone {
                overtone: 8.0,
                // overtone: 64.0,
                volume: 0.0,
            },
        ];
        let wave_table = WaveTables::new(&overtones).sin;
        let mut lfo = LFO::new();
        lfo.set_frequency(400.0 / 60.0);

        Self {
            osc_s: [Oscillator::new(); VOICES],
            wave_table,
            osc_type: OscType::Sin,
            overtones,
            lfo,
            volume: 1.0,
        }
    }

    pub fn set_overtones(&mut self) {
        self.wave_table = WaveTables::build_sine_table(&self.overtones);
    }

    pub fn get_sample(&mut self) -> f32 {
        let mut sample = 0.0;
        let lfo_sample = self.lfo.get_sample();
        // println!("lfo sample {lfo_sample}");

        for osc in self.osc_s.iter_mut() {
            // println!("{osc:?}");
            // for osc in osc_s {
            if osc.playing.is_some() {
                // osc.for_each(|(osc, _offset)| {
                osc.vibrato(lfo_sample);
                // println!("playing");
                sample += osc.get_sample(&self.wave_table);
                // println!(
                //     "env => {}, {}",
                //     osc.env_filter.get_samnple(),
                //     osc.env_filter.phase
                // );
                // });
            }
            // }
        }

        sample *= self.volume;
        sample += sample * lfo_sample * 0.25;
        sample.tanh()
    }

    pub fn play(&mut self, midi_note: MidiNote, _velocity: u8) {
        // let midi_note = if midi_note >= 12 {
        //     midi_note - 12
        // } else {
        //     return;
        // };

        // for (osc_s, _offset) in self.osc_s.iter_mut() {
        //     for osc in osc_s {
        for osc in self.osc_s.iter_mut() {
            if osc.playing == Some(midi_note) {
                return;
            }
        }
        // }

        // for (osc_s, offset) in self.osc_s.iter_mut() {
        //     for osc in osc_s {
        for osc in self.osc_s.iter_mut() {
            if osc.playing.is_none() {
                // let note = midi_note;
                // if *offset > 0 {
                //     midi_note + (*offset as u8)
                // } else {
                //     // println!("offset {} -> {}", offset, (offset.abs() as u8));
                //     midi_note - (offset.abs() as u8)
                // };
                osc.press(midi_note);
                osc.playing = Some(midi_note);
                // println!("playing note on osc {i}");

                break;
            }
        }
        // }
    }

    pub fn stop(&mut self, midi_note: MidiNote) {
        // let midi_note = if midi_note >= 12 {
        //     midi_note - 12
        // } else {
        //     return;
        // };

        // for (osc_s, _offset) in self.osc_s.iter_mut() {
        //     for osc in osc_s {
        for osc in self.osc_s.iter_mut() {
            // let note = if *offset > 0 {
            //     midi_note + (*offset as u8)
            // } else {
            //     // println!("offset {} -> {}", offset, (offset.abs() as u8));
            //     midi_note - (offset.abs() as u8)
            // };

            if osc.playing == Some(midi_note) {
                // println!("release");
                osc.release();
                break;
            }
        }
        // }
    }

    pub fn bend_all(&mut self, bend: f32) {
        // for (osc_s, _offset) in self.osc_s.iter_mut() {
        // for osc in osc_s {
        for osc in self.osc_s.iter_mut() {
            if osc.playing.is_some() {
                osc.bend(bend);
            }
        }
        // }
    }

    pub fn unbend(&mut self) {
        // for (osc_s, _offset) in self.osc_s.iter_mut() {
        //     for osc in osc_s {
        for osc in self.osc_s.iter_mut() {
            if osc.playing.is_some() {
                osc.unbend();
            }
        }
        // }
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol;
    }

    pub fn set_atk(&mut self, atk: f32) {
        // for (osc_s, _offset) in self.osc_s.iter_mut() {
        //     for osc in osc_s {
        for osc in self.osc_s.iter_mut() {
            osc.env_filter.set_atk(atk);
        }
        // }
    }

    pub fn set_decay(&mut self, decay: f32) {
        // for (osc_s, _offset) in self.osc_s.iter_mut() {
        //     for osc in osc_s {
        for osc in self.osc_s.iter_mut() {
            osc.env_filter.set_decay(decay);
        }
        // }
    }

    pub fn set_sus(&mut self, sus: f32) {
        // for (osc_s, _offset) in self.osc_s.iter_mut() {
        //     for osc in osc_s {
        for osc in self.osc_s.iter_mut() {
            osc.env_filter.set_sus(sus);
        }
        // }
    }

    pub fn set_release(&mut self, release: f32) {
        // for (osc_s, _offset) in self.osc_s.iter_mut() {
        //     for osc in osc_s {
        for osc in self.osc_s.iter_mut() {
            osc.env_filter.set_release(release);
        }
        // }
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        let cutoff = cutoff * 10_000.0;

        // for (osc_s, _offset) in self.osc_s.iter_mut() {
        //     for osc in osc_s {
        for osc in self.osc_s.iter_mut() {
            osc.low_pass.set_cutoff(cutoff);
        }
        // }
    }

    pub fn set_resonace(&mut self, resonace: f32) {
        // for (osc_s, _offset) in self.osc_s.iter_mut() {
        //     for osc in osc_s {
        for osc in self.osc_s.iter_mut() {
            osc.low_pass.set_resonace(resonace);
        }
        // }
    }

    // pub fn set_chorus_speed(&mut self, speed: f32) {
    //     self.chorus.set_speed(speed)
    // }
    //
    // pub fn set_chorus_depth(&mut self, depth: f32) {
    //     self.chorus.set_volume(depth)
    // }

    pub fn set_leslie_speed(&mut self, speed: f32) {
        self.lfo.set_frequency((400.0 * speed) / 60.0);
        self.lfo.set_volume(speed);
    }

    fn set_overtone(&mut self, overtone: usize, presence: f32) {
        self.overtones[overtone].volume = presence as f64;

        self.set_overtones();
    }

    // pub fn set_atk(&mut self, atk: f32) {}
}

impl SynthEngine for Organ {
    fn name(&self) -> String {
        "Organ".into()
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

    fn volume_swell(&mut self, amount: f32) -> bool {
        self.set_leslie_speed(amount);

        true
    }

    fn get_params(&mut self) -> HashMap<Knob, f32> {
        let mut map = HashMap::with_capacity(8);

        map.insert(Knob::One, self.overtones[0].volume as f32);
        map.insert(Knob::Two, self.overtones[1].volume as f32);
        map.insert(Knob::Three, self.overtones[2].volume as f32);
        map.insert(Knob::Four, self.overtones[3].volume as f32);
        map.insert(Knob::Five, self.overtones[4].volume as f32);
        map.insert(Knob::Six, self.overtones[5].volume as f32);
        map.insert(Knob::Seven, self.overtones[6].volume as f32);
        map.insert(Knob::Eight, self.overtones[7].volume as f32);

        map
    }

    fn get_gui_params(&mut self) -> HashMap<GuiParam, f32> {
        let mut map = HashMap::with_capacity(8);

        map.insert(GuiParam::A, self.osc_s[0].env_filter.base_params[ATTACK]);
        map.insert(GuiParam::B, self.osc_s[0].env_filter.base_params[DECAY]);
        map.insert(GuiParam::C, self.osc_s[0].env_filter.base_params[SUSTAIN]);
        map.insert(GuiParam::D, self.osc_s[0].env_filter.base_params[RELEASE]);
        map.insert(GuiParam::E, self.lfo.volume);

        map
    }
}

impl SampleGen for Organ {
    fn get_sample(&mut self) -> f32 {
        self.get_sample()
    }
}

impl KnobCtrl for Organ {
    fn knob_1(&mut self, value: f32) -> bool {
        // self.set_atk(value);
        // // update_callback(SynthParam::Atk(value));
        // Some(SynthParam::Atk(value))
        self.set_overtone(0, value);

        true
    }

    fn knob_2(&mut self, value: f32) -> bool {
        // self.set_decay(value);
        // // update_callback(SynthParam::Dcy(value));
        // Some(SynthParam::Dcy(value))
        self.set_overtone(1, value);

        true
    }

    fn knob_3(&mut self, value: f32) -> bool {
        // self.set_sus(value);
        // // update_callback(SynthParam::Sus(value));
        // Some(SynthParam::Sus(value))
        self.set_overtone(2, value);

        true
    }

    fn knob_4(&mut self, value: f32) -> bool {
        // self.set_release(value);
        // // update_callback(SynthParam::Rel(value));
        // Some(SynthParam::Rel(value))
        self.set_overtone(3, value);

        true
    }

    fn knob_5(&mut self, value: f32) -> bool {
        // self.set_leslie_speed(value);
        // // update_callback(SynthParam::SpeakerSpinSpeed(value));
        // Some(SynthParam::SpeakerSpinSpeed(value))
        self.set_overtone(4, value);

        true
    }

    fn knob_6(&mut self, value: f32) -> bool {
        self.set_overtone(5, value);
        true
    }

    fn knob_7(&mut self, value: f32) -> bool {
        self.set_overtone(6, value);
        true
    }

    fn knob_8(&mut self, value: f32) -> bool {
        self.set_overtone(7, value);
        true
    }

    fn gui_param_1(&mut self, value: f32) -> bool {
        self.set_atk(value);
        true
    }

    fn gui_param_2(&mut self, value: f32) -> bool {
        self.set_decay(value);
        true
    }

    fn gui_param_3(&mut self, value: f32) -> bool {
        self.set_sus(value);
        true
    }

    fn gui_param_4(&mut self, value: f32) -> bool {
        self.set_release(value);
        true
    }

    fn gui_param_5(&mut self, value: f32) -> bool {
        self.set_leslie_speed(value);

        true
    }
}