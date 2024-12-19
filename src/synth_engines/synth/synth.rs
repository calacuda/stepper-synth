use crate::{
    pygame_coms::{GuiParam, Knob},
    synth_engines::{
        synth_common::{
            env::{ATTACK, DECAY, RELEASE, SUSTAIN},
            osc::{Oscillator, Overtone},
        },
        SynthEngine,
    },
    KnobCtrl, SampleGen,
};
use midi_control::MidiNote;
use pyo3::prelude::*;
use std::{collections::HashMap, sync::Arc};

pub type WaveTable = Arc<[f32]>;
// pub type WaveTables = [(WaveTable, f32); 2];

pub const WAVE_TABLE_SIZE: usize = 256;
pub const VOICES: usize = 10;

#[pyclass(module = "stepper_synth_backend", get_all, eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum OscType {
    Sin,
    Tri,
    Sqr,
    Saw,
}

impl From<usize> for OscType {
    fn from(value: usize) -> Self {
        match value {
            _ if value == Self::Sin as usize => Self::Sin,
            _ if value == Self::Tri as usize => Self::Tri,
            _ if value == Self::Sqr as usize => Self::Sqr,
            _ if value == Self::Saw as usize => Self::Saw,
            _ => Self::Saw,
        }
    }
}

#[pymethods]
impl OscType {
    #[new]
    fn new(f: f64) -> Self {
        Self::from(f as usize)
    }

    fn __str__(&self) -> String {
        match *self {
            OscType::Sin => "Sin".into(),
            OscType::Tri => "Tri".into(),
            OscType::Sqr => "Sqr".into(),
            OscType::Saw => "Saw".into(),
        }
    }
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

        let n_overtones = overtones.len();

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

    fn index(&self, index: &Arc<[(OscType, f32)]>) -> Arc<[(WaveTable, f32)]> {
        index
            .iter()
            .map(|(osc_type, vol)| {
                (
                    match osc_type {
                        OscType::Sin => self.sin.clone(),
                        OscType::Tri => self.tri.clone(),
                        OscType::Sqr => self.sqr.clone(),
                        OscType::Saw => self.saw.clone(),
                    },
                    vol / index.len() as f32,
                )
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Synth {
    pub osc_s: [([Oscillator; VOICES], i16); 2],
    pub wave_tables: WaveTables,
    pub osc_type: [(OscType, f32); 2],
    pub overtones: [Overtone; 10],
    pub volume: f32,
    pub mix: f32,
}

impl Synth {
    pub fn new() -> Self {
        let overtones = [
            // Overtone {
            //     overtone: 0.5_f64.powf(1.0 / 12.0),
            //     volume: 1.0,
            // },
            // Overtone {
            //     // overtone: 2.0_f64.powf(1.0 / 12.0),
            //     overtone: 1.5_f64.powf(1.0 / 12.0),
            //     volume: 1.0,
            // },
            Overtone {
                overtone: 1.0,
                volume: 1.0,
            },
            Overtone {
                overtone: 2.0,
                volume: 1.0,
            },
            Overtone {
                overtone: 3.0,
                // overtone: 4.0,
                volume: 1.0,
            },
            Overtone {
                overtone: 4.0,
                // overtone: 8.0,
                volume: 1.0,
            },
            Overtone {
                overtone: 5.0,
                // overtone: 16.0,
                volume: 1.0,
            },
            Overtone {
                overtone: 6.0,
                // overtone: 32.0,
                volume: 1.0,
            },
            Overtone {
                overtone: 7.0,
                // overtone: 32.0,
                volume: 1.0,
            },
            Overtone {
                overtone: 8.0,
                // overtone: 64.0,
                volume: 1.0,
            },
            Overtone {
                overtone: 9.0,
                // overtone: 128.0,
                volume: 1.0,
            },
            Overtone {
                overtone: 10.0,
                // overtone: 256.0,
                volume: 1.0,
            },
            // Overtone {
            //     overtone: 11.0,
            //     // overtone: 256.0,
            //     volume: 1.0,
            // },
        ];
        let wave_tables = WaveTables::new(&overtones);

        Self {
            osc_s: [([Oscillator::new(); VOICES], 0); 2],
            wave_tables,
            osc_type: [
                // (OscType::Sin, 1.0),
                (OscType::Sin, 1.0),
                (OscType::Saw, 1.0),
                // (OscType::Saw, 1.0),
                // (OscType::Tri, 0.75),
                // (OscType::Sqr, 1.0),
            ],
            overtones,
            // osc_type: Arc::new([(OscType::Tri, 1.0)]),
            volume: 0.75,
            mix: 0.5,
        }
    }

    pub fn set_overtones(&mut self) {
        self.wave_tables = WaveTables::new(&self.overtones);
    }

    pub fn get_sample(&mut self) -> f32 {
        let mut sample = 0.0;
        // println!("lfo sample {lfo_sample}");

        for (((osc_s, _offset), (wave_table, volume)), mix) in self
            .osc_s
            .iter_mut()
            .zip(self.wave_tables.index(&self.osc_type.clone().into()).iter())
            .zip([1.0 - self.mix, self.mix])
        {
            // println!("{:?}", osc_s.len());
            // info!("mix :  {mix}");
            for osc in osc_s {
                if osc.playing.is_some() {
                    // osc.for_each(|(osc, _offset)| {
                    // info!("mix :  {mix}");

                    // println!("playing");
                    sample += osc.get_sample(&wave_table) * volume * mix;
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
        let cutoff = cutoff * 16_000.0;

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
        map.insert(Knob::Five, self.osc_s[0].0[0].low_pass.cutoff / 16_000.0);
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
        true
    }

    fn gui_param_2(&mut self, value: f32) -> bool {
        self.osc_type[1].0 = OscType::from(value as usize);
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
