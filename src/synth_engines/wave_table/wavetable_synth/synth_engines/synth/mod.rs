use super::super::{config::OSC_WAVE_TABLE_SIZE, sin, OscWaveTable};

pub const N_OVERTONES_SAW: usize = 16;

pub mod osc;

pub fn build_sine_table(overtones: &[f64]) -> OscWaveTable {
    let mut wave_table = [0.0; OSC_WAVE_TABLE_SIZE];

    let n_overtones = overtones.len();

    let bias = 1.0 / (n_overtones as f32 * 0.5);

    for i in 0..OSC_WAVE_TABLE_SIZE {
        for ot in overtones {
            wave_table[i] +=
                sin(2.0 * core::f64::consts::PI * i as f64 * ot / OSC_WAVE_TABLE_SIZE as f64) as f32
        }

        wave_table[i] *= bias;
    }

    wave_table.into()
}
