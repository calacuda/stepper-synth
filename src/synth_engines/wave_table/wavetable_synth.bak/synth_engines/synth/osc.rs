use crate::synth_engines::wave_table::wavetable_synth::{
    calculate_modulation,
    common::OscParam,
    config::{OSC_WAVE_TABLE_SIZE, SAMPLE_RATE},
    midi_to_freq, pow, ModulationDest, OscWaveTable, SampleGen,
};
// use libm::powf;
#[allow(unused_imports)]
use log::*;

pub const N_OVERTONES: usize = 32;

#[derive(Clone, Copy, Debug)]
pub enum OscTarget {
    Filter1,
    Filter2,
    Filter1_2,
    // Effects,
    DirectOut,
}

#[derive(Clone, Copy, Debug)]
pub struct WavetableOscillator {
    sample_rate: f32,
    index: f32,
    index_increment: f32,
}

impl WavetableOscillator {
    pub fn new() -> Self {
        Self {
            sample_rate: SAMPLE_RATE as f32,
            index: 0.0,
            index_increment: 0.0,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.index_increment = frequency * OSC_WAVE_TABLE_SIZE as f32 / self.sample_rate;
        self.index = 0.0;
    }

    pub fn get_sample(&mut self, wave_table: &[f32]) -> f32 {
        let mut sample = 0.0;

        sample += self.lerp(wave_table);

        self.index += self.index_increment;
        self.index %= OSC_WAVE_TABLE_SIZE as f32;

        sample * 0.9
    }

    fn lerp(&self, wave_table: &[f32]) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % OSC_WAVE_TABLE_SIZE;

        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;

        truncated_index_weight * wave_table[truncated_index]
            + next_index_weight * wave_table[next_index]
    }
}

#[derive(Clone, Debug)]
pub struct Oscillator {
    osc: WavetableOscillator,
    frequency: f32,
    base_frequency: f32,
    pub level: f32,
    level_mod: f32,
    // pan: f32,
    pub detune: f32,
    detune_mod: f32,
    pub offset: i16,
    pub target: OscTarget,
    pub wave_table: OscWaveTable,
}

impl Oscillator {
    pub fn new(wave_table: OscWaveTable) -> Self {
        Self {
            osc: WavetableOscillator::new(),
            frequency: 0.0,
            base_frequency: 0.0,
            level: 1.0,
            level_mod: 0.0,
            detune: 0.0,
            detune_mod: 0.0,
            offset: 0,
            target: OscTarget::Filter1_2,
            wave_table,
        }
    }

    pub fn press(&mut self, midi_note: u8) {
        let note = midi_note as i16 + self.offset;

        self.frequency = midi_to_freq(note);
        // warn!("midi note: {note}|{midi_note} => {}", self.frequency);
        self.base_frequency = self.frequency;

        self.osc.set_frequency(self.frequency);
    }

    pub fn release(&mut self) {}

    pub fn get_sample(&mut self) -> f32 {
        // if self.tune != 0.0 {
        // self.detune(tune)
        // }
        self.detune();

        // info!(
        //     "modulated volume {}, {}",
        //     calculate_modulation(self.level, self.level_mod),
        //     self.level_mod,
        // );

        self.osc.get_sample(&self.wave_table)
            * calculate_modulation(self.level, self.level_mod).tanh()
    }

    pub fn detune(&mut self) {
        // println!("bending");
        if self.detune == 0.0 && self.detune_mod == 0.0 {
            return;
        }

        let amt = calculate_modulation(self.detune, self.detune_mod);

        if amt == 0.0 {
            return;
        };

        // let nudge = 2.0_f32.powf(amt / 12.0);
        let nudge = pow(2.0, amt / 12.0);
        let new_freq = if amt < 0.0 {
            self.frequency / nudge
        } else if amt > 0.0 {
            self.frequency * nudge
        } else {
            self.frequency
        };
        // + self.frequency;
        self.osc.set_frequency(new_freq);
        // println!("frequency => {}", self.frequency);
        // println!("new_freq => {}", new_freq);
        self.frequency = new_freq;
    }

    pub fn bend(&mut self, bend: f32) {
        // println!("bending");
        // let nudge = 2.0_f32.powf((bend * 3.0) / 12.0);
        let nudge = pow(2.0, (bend * 3.0) / 12.0);
        let new_freq = if bend < 0.0 {
            self.base_frequency / nudge
        } else if bend > 0.0 {
            self.base_frequency * nudge
        } else {
            self.base_frequency
        };
        // + self.frequency;
        self.osc.set_frequency(new_freq);
        // println!("frequency => {}", self.frequency);
        // println!("new_freq => {}", new_freq);
        self.frequency = new_freq;
    }

    pub fn unbend(&mut self) {
        // println!("unbend => {}", self.base_frequency);
        self.osc.set_frequency(self.base_frequency);
        self.frequency = self.base_frequency;
    }
}

impl ModulationDest for Oscillator {
    type ModTarget = OscParam;

    fn modulate(&mut self, what: Self::ModTarget, by: f32) {
        // info!("modulating {what:?} by {by}");

        match what {
            Self::ModTarget::Level => self.level_mod = by,
            Self::ModTarget::Tune => self.detune_mod = by,
        }

        // info!("{}", self.level_mod);
    }

    fn reset(&mut self) {
        // info!("resseting");
        // info!("{}", self.level_mod);
        // info!(
        //     "modulated volume {}, {}",
        //     calculate_modulation(self.level, self.level_mod),
        //     self.level_mod,
        // );

        self.level_mod = 0.0;
        self.detune_mod = 0.0;
    }
}

impl SampleGen for Oscillator {
    fn get_sample(&mut self) -> f32 {
        self.get_sample()
    }
}
