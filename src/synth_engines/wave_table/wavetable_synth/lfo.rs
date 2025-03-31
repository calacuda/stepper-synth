use super::{
    calculate_modulation,
    common::LfoParam,
    config::{LFO_WAVE_TABLE_SIZE, SAMPLE_RATE},
    LfoWaveTable, ModulationDest, SampleGen,
};

/// the WaveTable oscillator that is used for generating LFO samples
#[derive(Clone, Debug)]
pub struct LfoWaveTableOsc {
    sample_rate: f32,
    index: f32,
    index_increment: f32,
    wave_table_len: f32,
    freq: f32,
}

impl LfoWaveTableOsc {
    pub fn new() -> Self {
        Self {
            sample_rate: SAMPLE_RATE as f32,
            index: 0.0,
            index_increment: 0.0,
            wave_table_len: LFO_WAVE_TABLE_SIZE as f32,
            freq: 2.0,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.freq = frequency;
        self.calc_index_inc();
    }

    fn calc_index_inc(&mut self) {
        self.index_increment = self.freq * self.wave_table_len / self.sample_rate;
    }

    pub fn set_wave_table_size(&mut self, size: usize) {
        self.wave_table_len = size as f32;
        self.calc_index_inc();
        self.index = 0.0;
    }

    pub fn get_sample(&mut self, wave_table: &[f32]) -> f32 {
        let sample = self.lerp(wave_table);

        self.index += self.index_increment;
        self.index %= self.wave_table_len;

        sample * 0.95
    }

    pub fn press(&mut self) {
        self.index = 0.0;
    }

    fn lerp(&self, wave_table: &[f32]) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % LFO_WAVE_TABLE_SIZE;

        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;

        truncated_index_weight * wave_table[truncated_index]
            + next_index_weight * wave_table[next_index]
    }
}

/// the actual LFO struct
#[derive(Clone, Debug)]
pub struct LFO {
    /// can be modulated by envelopes, lfos, velocity, etc
    pub freq: f32,
    speed_mod: f32,
    wave_table: LfoWaveTable,
    pub osc: LfoWaveTableOsc,
    playing: bool,
}

impl LFO {
    pub fn new() -> Self {
        let wave_table = mk_default_lfo_wt();

        Self {
            freq: 2.0,
            speed_mod: 0.0,
            wave_table,
            osc: LfoWaveTableOsc::new(),
            playing: false,
        }
    }

    pub fn get_sample(&mut self) -> f32 {
        if self.playing {
            self.osc.get_sample(&self.wave_table)
        } else {
            0.0
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.freq = frequency;
        self.osc.set_frequency(frequency);
    }

    pub fn set_wave_table(&mut self, wave_table: LfoWaveTable) {
        self.wave_table = wave_table;
        self.osc.set_wave_table_size(self.wave_table.len());
    }

    pub fn press(&mut self) {
        self.osc.press();
        self.osc.index = 0.0;
        self.playing = true;
    }

    pub fn release(&mut self) {
        self.playing = false;
    }
}

impl SampleGen for LFO {
    fn get_sample(&mut self) -> f32 {
        self.osc.get_sample(&self.wave_table)
    }
}

impl ModulationDest for LFO {
    type ModTarget = LfoParam;

    fn modulate(&mut self, what: Self::ModTarget, by: f32) {
        match what {
            Self::ModTarget::Speed => {
                // if self.speed_mod != by {
                self.speed_mod = by;
                self.osc.set_frequency(calculate_modulation(self.freq, by))
                // }
            }
        }
    }

    fn reset(&mut self) {
        self.speed_mod = 0.0;
        self.osc.set_frequency(self.freq);
    }
}

fn mk_default_lfo_wt() -> LfoWaveTable {
    let size = LFO_WAVE_TABLE_SIZE;
    let half_way = size as f32 / 2.0;
    let slope = 2.0 / size as f32;
    let f = |i: usize| slope * i as f32;

    let mut table = [0.0; LFO_WAVE_TABLE_SIZE];

    for i in 0..size {
        table[i] = if i as f32 <= half_way {
            f(i)
        } else {
            1.0 - f(i)
        };
    }

    table
}
