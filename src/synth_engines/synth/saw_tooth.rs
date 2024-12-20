use crate::{SampleGen, SAMPLE_RATE};

use super::{osc::N_OVERTONES, SynthOscilatorBackend};

#[derive(Debug, Clone, Copy, Default)]
struct STOsc {
    value: f64,
    inc: f64,
}

impl SampleGen for STOsc {
    fn get_sample(&mut self) -> f32 {
        self.value += self.inc;
        self.value %= 2.0;

        (self.value - 1.0) as f32
    }
}

impl SynthOscilatorBackend for STOsc {
    fn set_frequency(&mut self, frequency: f32) {
        let n_peeks = frequency as f64 * 2.0;
        self.inc = 1.0 / (SAMPLE_RATE as f64 / n_peeks);
    }
}

#[derive(Debug, Clone)]
pub struct SawToothOsc {
    osc_s: [STOsc; N_OVERTONES],
}

impl SawToothOsc {
    pub fn new() -> Self {
        let mut osc = STOsc::default();
        osc.value = 1.0;

        Self {
            osc_s: [osc; N_OVERTONES],
        }
    }
}

impl SampleGen for SawToothOsc {
    fn get_sample(&mut self) -> f32 {
        let mut sample = 0.0;

        for ref mut osc in self.osc_s.iter_mut() {
            sample += osc.get_sample();
        }

        sample / (N_OVERTONES as f32 * 0.5)
    }
}

impl SynthOscilatorBackend for SawToothOsc {
    fn set_frequency(&mut self, frequency: f32) {
        for (i, ref mut osc) in self.osc_s.iter_mut().enumerate() {
            osc.set_frequency(frequency * i as f32 + frequency)
        }
    }
}
