use super::{
    build_sine_table, saw_tooth::SawToothOsc, OscType, SynthBackend, SynthOscilatorBackend,
};
use crate::{
    synth_engines::synth_common::{env::ADSR, moog_filter::LowPass, WaveTable, WAVE_TABLE_SIZE},
    SampleGen, SAMPLE_RATE,
};

pub const N_OVERTONES: usize = 10;

#[derive(Clone, Debug)]
pub struct WavetableOscillator {
    sample_rate: f32,
    index: f32,
    index_increment: f32,
    wave_table: WaveTable,
    direction: bool,
}

impl WavetableOscillator {
    pub fn new() -> Self {
        let overtones: Vec<f64> = (1..=N_OVERTONES).map(|i| i as f64).collect();
        let wave_table = build_sine_table(&overtones);

        Self {
            sample_rate: SAMPLE_RATE as f32,
            index: 0.0,
            index_increment: 0.0,
            wave_table,
            direction: true,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.index_increment = frequency * WAVE_TABLE_SIZE as f32 / self.sample_rate;
        self.index = 0.0;
    }

    fn get_sample(&mut self) -> f32 {
        let mut sample = 0.0;

        // for table in wave_tables.iter() {
        sample += self.lerp(&self.wave_table);
        // }

        if self.direction {
            self.index += self.index_increment;
            self.index %= WAVE_TABLE_SIZE as f32;
        }
        // else if self.index_increment > self.index {
        //     let inc = self.index_increment - self.index;
        //     self.index = WAVE_TABLE_SIZE as f32 - 1.0 - inc;
        // }
        else {
            self.index -= self.index_increment;
            self.index %= WAVE_TABLE_SIZE as f32;
        }

        sample
    }

    fn lerp(&self, wave_table: &[f32]) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % WAVE_TABLE_SIZE;

        // if next_index == WAVE_TABLE_SIZE {
        //     next_index = 0;
        // }

        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;

        truncated_index_weight * wave_table[truncated_index]
            + next_index_weight * wave_table[next_index]
    }
}

impl SampleGen for WavetableOscillator {
    fn get_sample(&mut self) -> f32 {
        self.get_sample()
    }
}

impl SynthOscilatorBackend for WavetableOscillator {
    fn set_frequency(&mut self, frequency: f32) {
        self.set_frequency(frequency)
    }

    fn sync_reset(&mut self) {
        if self.index > WAVE_TABLE_SIZE as f32 * (3.0 / 12.0)
        // && self.wave_table[self.index as usize] != 0.0
        {
            // warn!("reset wave_table");
            // self.index = 0.0;
            self.direction = !self.direction;
        }
    }
}

#[derive(Debug, Clone)]
pub struct SynthOscillator {
    osc: SynthBackend,
    pub env_filter: ADSR,
    /// what midi note is being played by this osc
    pub playing: Option<u8>,
    frequency: f32,
    base_frequency: f32,
    // note_space: f32,
    pub low_pass: LowPass,
    // pub wave_table: WaveTable,
}

impl SynthOscillator {
    pub fn new() -> Self {
        Self {
            // osc: SynthBackend::Sin(WavetableOscillator::new()),
            osc: SynthBackend::Saw(SawToothOsc::new()),
            env_filter: ADSR::new(),
            playing: None,
            frequency: 0.0,
            base_frequency: 0.0,
            // note_space: 2.0_f32.powf(1.0 / 12.0),
            low_pass: LowPass::new(),
        }
    }

    pub fn sync_reset(&mut self) {
        self.osc.sync_reset()
    }

    pub fn set_osc_type(&mut self, osc_type: OscType) {
        self.osc = osc_type.into();
    }

    pub fn is_pressed(&self) -> bool {
        self.env_filter.pressed()
    }

    pub fn press(&mut self, midi_note: u8) {
        self.env_filter.press();
        self.frequency = Self::get_freq(midi_note);
        self.base_frequency = self.frequency;

        self.osc.set_frequency(self.frequency);
        self.low_pass.set_note(self.frequency);
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

    pub fn get_sample(&mut self) -> f32 {
        let env = self.env_filter.get_samnple();
        let sample = self.osc.get_sample() * env;

        if env <= 0.0 {
            self.playing = None;
        }
        // println!("osc sample => {sample}");

        self.low_pass.get_sample(sample, env)
    }

    pub fn bend(&mut self, bend: f32) {
        // println!("bending");
        let nudge = 2.0_f32.powf((bend * 3.0).abs() / 12.0);
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
