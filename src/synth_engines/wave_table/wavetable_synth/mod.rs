// use crate::effects::chorus::Chorus;
// use crate::effects::reverb::Reverb;
// use crate::effects::EffectsModule;
pub use crate::{MidiControlled, SampleGen};
use array_macro::array;
use common::DataTable;
use common::ModMatrixDest;
use common::ModMatrixItem;
use config::LFO_WAVE_TABLE_SIZE;
use config::N_LFO;
use config::OSC_WAVE_TABLE_SIZE;
use config::POLYPHONY;
use lfo::LFO;
#[allow(unused_imports)]
use log::*;
use synth_engines::synth::build_sine_table;
use synth_engines::synth::osc::N_OVERTONES;
use voice::Voice;

// pub type HashMap<Key, Val> = fxhash::FxHashMap<Key, Val>;
pub type ModMatrix = [Option<ModMatrixItem>; 255];
pub type OscWaveTable = [f32; OSC_WAVE_TABLE_SIZE];
pub type LfoWaveTable = [f32; LFO_WAVE_TABLE_SIZE];

pub mod common;
pub mod config;
// pub mod effects;
pub mod lfo;
pub mod synth_engines;
pub mod voice;

pub trait ModulationDest {
    type ModTarget;

    fn modulate(&mut self, what: Self::ModTarget, by: f32);
    /// clears any applied modulation.
    fn reset(&mut self);
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct App {
    /// describes what modulates what.
    pub mod_matrix: ModMatrix,
    /// used for routung cc messages
    pub midi_table: [Option<ModMatrixDest>; 255],
    /// the sound producers
    pub voices: Vec<Voice>,
    /// LFOs
    pub lfos: [LFO; N_LFO],
    /// holds the out put of the different modules and also other needed data (velocity, and note).
    data_table: DataTable,
}

impl Default for App {
    fn default() -> Self {
        let mut overtones = [1.0; N_OVERTONES];

        (0..N_OVERTONES).for_each(|i| overtones[i] = (i + 1) as f64);

        let wave_table = build_sine_table(&overtones);

        let voices = (0..POLYPHONY)
            .map(|_| Voice::new(wave_table.clone()))
            .collect();

        Self {
            mod_matrix: [None; 255],
            data_table: DataTable::default(),
            midi_table: [None; 255],
            lfos: array![LFO::new(); N_LFO],
            voices,
        }
    }
}

#[allow(unused_variables)]
impl MidiControlled for App {
    fn midi_input(&mut self, message: &midi_control::MidiMessage) {
        use midi_control::{KeyEvent, MidiMessage};

        // TODO: if note, add midi note to the data table
        // TODO: if cc, route based on learned midi table
        match *message {
            MidiMessage::NoteOn(_channel, KeyEvent { key, value }) => self.play(key, value),
            MidiMessage::NoteOff(_channel, KeyEvent { key, value }) => self.stop(key),
            _ => {}
        }
    }
}

impl SampleGen for App {
    fn get_sample(&mut self) -> f32 {
        for mod_entry in self.mod_matrix {
            if let Some(entry) = mod_entry {
                // get mod amount
                let mut amt = self.data_table.get_entry(&entry.src) * entry.amt;

                if entry.bipolar {
                    amt -= entry.amt / 2.0;
                }

                match entry.dest {
                    ModMatrixDest::Lfo { lfo, param } => self.lfos[lfo].modulate(param, amt),
                    _ => {}
                }
            }
        }

        // calculate lfos
        if !self.data_table.osc_order.is_empty() {
            for (i, lfo) in self.lfos.iter_mut().enumerate() {
                let sample = lfo.get_sample();
                self.data_table.lfos[i] = sample;
            }
        }

        let sample: f32 = self
            .voices
            .iter_mut()
            .filter_map(|voice| voice.get_sample(&self.mod_matrix, &mut self.data_table))
            .sum();

        // TODO: add an AllPass filter

        sample.tanh()
    }
}

impl App {
    pub fn play(&mut self, note: midi_control::MidiNote, velocity: u8) {
        // for voice in self.voices.iter_mut() {
        //     // let mut voice = voice.lock().unwrap();
        //
        //     if voice.playing.is_none() {
        //         // info!("playing note {note}");
        //         voice.press(note, velocity);
        //         break;
        //     }
        // }
        for (i, voice) in self.voices.iter_mut().enumerate() {
            // let mut voice = voice.lock().unwrap();

            if voice.playing.is_none() {
                // info!("playing note {key}");
                voice.press(note, velocity);
                // self.data_table.velocity = Some(value);
                // self.data_table.note = Some(key);
                self.data_table.note[i] = Some((note, velocity));
                self.data_table.osc_order.push(i);
                self.lfos.iter_mut().for_each(|lfo| {
                    lfo.release();
                    lfo.reset();
                    lfo.press();
                });

                break;
            }
        }
    }

    pub fn stop(&mut self, note: midi_control::MidiNote) {
        // for voice in self.voices.iter() {
        //     let mut voice = voice.lock().unwrap();
        //
        //     if voice.playing.is_some_and(|n| n == note) {
        //         voice.release();
        //     }
        // }
        for (i, voice) in self.voices.iter_mut().enumerate() {
            // let mut voice = voice.lock().unwrap();

            if voice.playing.is_some_and(|playing| playing == note) {
                voice.release();
                self.data_table.note[i] = None;
                self.data_table.osc_order.retain(|osc| *osc != i);
            }
        }

        if self.data_table.osc_order.is_empty() {
            self.lfos.iter_mut().for_each(|lfo| lfo.press());
        }
    }
}

pub fn midi_to_freq(midi_note: i16) -> f32 {
    let exp = (f32::from(midi_note) + 36.376_316) / 12.0;

    pow(2.0, exp)
}

pub fn calculate_modulation(base: f32, amt: f32) -> f32 {
    base + base * amt
}

#[inline]
fn pow(base: f32, exp: f32) -> f32 {
    base.powf(exp)
}

#[inline]
fn tanh(x: f32) -> f32 {
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;

    let a = x + (0.16489087 * x3) + (0.00985468 * x5);

    a / (1.0 + (a * a)).sqrt()
}

#[inline]
fn exp(x: f32) -> f32 {
    x.exp()
}

#[inline]
fn sin(x: f64) -> f64 {
    x.sin()
}
