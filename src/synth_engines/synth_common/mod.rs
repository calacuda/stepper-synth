use std::sync::Arc;

pub mod env;
pub mod lfo;
pub mod moog_filter;
pub mod osc;

pub type WaveTable = Arc<[f32]>;
pub const WAVE_TABLE_SIZE: usize = 128;
