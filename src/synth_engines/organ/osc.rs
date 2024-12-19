// use super::{env::ADSR, moog_filter::LowPass};
use crate::synth_engines::synth_common::{env::ADSR, osc::WavetableOscillator, WaveTable};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct Overtone {
    /// the frequency of the overtone relative to the fundimental
    pub overtone: f64,
    /// how loud this over tone is relative to the total volume (ie, 1.0)
    pub volume: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Oscillator {
    wt_osc: WavetableOscillator,
    pub env_filter: ADSR,
    /// what midi note is being played by this osc
    pub playing: Option<u8>,
    frequency: f32,
    base_frequency: f32,
    note_space: f32,
    // pub low_pass: LowPass,
}

impl Oscillator {
    pub fn new() -> Self {
        Self {
            wt_osc: WavetableOscillator::new(),
            env_filter: ADSR::new(),
            playing: None,
            frequency: 0.0,
            base_frequency: 0.0,
            note_space: 2.0_f32.powf(1.0 / 12.0),
            // low_pass: LowPass::new(),
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.env_filter.pressed()
    }

    pub fn press(&mut self, midi_note: u8) {
        self.env_filter.press();
        self.frequency = Self::get_freq(midi_note);
        self.base_frequency = self.frequency;

        self.wt_osc.set_frequency(self.frequency);
        self.playing = Some(midi_note);
    }

    fn get_freq(midi_note: u8) -> f32 {
        let exp = (f32::from(midi_note) + 36.376_316) / 12.0;
        // 2_f32.powf(exp)

        2.0_f32.powf(exp)
    }

    pub fn release(&mut self) {
        self.env_filter.release();
        // self.playing = None;
    }

    pub fn get_samples(&mut self, wave_table: &Arc<[(WaveTable, f32)]>) -> f32 {
        let env = self.env_filter.get_samnple();
        let sample = self.wt_osc.get_samples(wave_table) * env;

        if env <= 0.0 {
            self.playing = None;
        }
        // println!("osc sample => {sample}");

        sample * env
    }

    pub fn get_sample(&mut self, wave_table: &WaveTable) -> f32 {
        let env = self.env_filter.get_samnple();
        let sample = self.wt_osc.get_sample(wave_table) * env;

        if env <= 0.0 {
            self.playing = None;
        }

        // self.low_pass.get_sample(sample, env)
        sample * env
    }

    pub fn vibrato(&mut self, amt: f32) {
        let amt = amt * 0.4;

        let next_note = if amt > 0.0 {
            self.frequency * self.note_space
        } else if amt == 0.0 {
            self.wt_osc.set_frequency(self.frequency);
            return;
        } else {
            self.frequency / self.note_space
        };

        let freq_delta = (self.frequency - next_note).abs();
        let adjust_amt = freq_delta * amt * 0.5;
        self.wt_osc.set_frequency(self.frequency + adjust_amt)
    }

    pub fn bend(&mut self, bend: f32) {
        // println!("bending");
        let new_freq = self.base_frequency * 2.0_f32.powf((bend * 3.0) / 12.0);
        // + self.frequency;
        self.wt_osc.set_frequency(new_freq);
        // println!("frequency => {}", self.frequency);
        // println!("new_freq => {}", new_freq);
        self.frequency = new_freq;
    }

    pub fn unbend(&mut self) {
        // println!("unbend => {}", self.base_frequency);
        self.wt_osc.set_frequency(self.base_frequency);
        self.frequency = self.base_frequency;
    }

    // pub fn set_wave_table(&mut )
}

// impl SampleGen for Oscillator {
//     fn get_sample(&mut self) -> f32 {
//         self.get_sample()
//     }
// }
