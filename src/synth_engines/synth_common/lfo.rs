use crate::SAMPLE_RATE;

pub const LFO_WAVE_TABLE_SIZE: usize = 128;

#[derive(Clone, Copy, Debug)]
pub struct LFO {
    sample_rate: u32,
    wave_table: [f32; LFO_WAVE_TABLE_SIZE],
    index: f32,
    index_increment: f32,
    volume: f32,
}

impl LFO {
    pub fn new() -> Self {
        Self {
            sample_rate: SAMPLE_RATE,
            wave_table: Self::build_wave_table(),
            index: 0.0,
            index_increment: 0.0,
            volume: 1.0,
        }
    }

    fn build_wave_table() -> [f32; LFO_WAVE_TABLE_SIZE] {
        let mut wave_table = [0.0; LFO_WAVE_TABLE_SIZE];

        for i in 0..LFO_WAVE_TABLE_SIZE {
            wave_table[i] =
                (2.0 * core::f64::consts::PI * i as f64 / LFO_WAVE_TABLE_SIZE as f64).sin() as f32
        }

        wave_table
    }
    pub fn set_frequency(&mut self, frequency: f32) {
        self.index_increment = frequency * LFO_WAVE_TABLE_SIZE as f32 / self.sample_rate as f32;
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    pub fn get_sample(&mut self) -> f32 {
        let sample = self.lerp();
        self.index += self.index_increment;
        self.index %= LFO_WAVE_TABLE_SIZE as f32;
        sample * self.volume
    }

    fn lerp(&self) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % LFO_WAVE_TABLE_SIZE;

        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;

        truncated_index_weight * self.wave_table[truncated_index]
            + next_index_weight * self.wave_table[next_index]
    }
}
