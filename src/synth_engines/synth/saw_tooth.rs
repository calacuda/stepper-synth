use super::{SynthOscilatorBackend, N_OVERTONES_SAW};
use crate::{SampleGen, SAMPLE_RATE};

#[derive(Debug, Clone, Copy, Default)]
struct STOsc {
    value: f64,
    inc: f64,
    dir: bool,
}

impl SampleGen for STOsc {
    fn get_sample(&mut self) -> f32 {
        if !self.dir {
            self.value += self.inc;
        } else {
            self.value -= self.inc;
        }
        self.value %= 2.0;

        (self.value - 1.0) as f32
    }
}

impl SynthOscilatorBackend for STOsc {
    fn set_frequency(&mut self, frequency: f32) {
        let n_peeks = frequency as f64 * 2.0;
        self.inc = 2.0 / (SAMPLE_RATE as f64 / n_peeks);
        self.value = -1.0;
    }

    fn sync_reset(&mut self) {
        // warn!("reset");
        // self.value = 1.0;
        self.dir = !self.dir;
    }
}

#[derive(Debug, Clone)]
pub struct SawToothOsc {
    osc_s: [STOsc; N_OVERTONES_SAW],
}

impl SawToothOsc {
    pub fn new() -> Self {
        let mut osc = STOsc::default();
        osc.value = 1.0;

        Self {
            osc_s: [osc; N_OVERTONES_SAW],
        }
    }
}

impl SampleGen for SawToothOsc {
    fn get_sample(&mut self) -> f32 {
        let mut sample = 0.0;

        for ref mut osc in self.osc_s.iter_mut() {
            sample += osc.get_sample();
        }

        // sample / (N_OVERTONES_SAW as f32 * 0.25)
        sample * 0.96
    }
}

impl SynthOscilatorBackend for SawToothOsc {
    fn set_frequency(&mut self, frequency: f32) {
        for (i, ref mut osc) in self.osc_s.iter_mut().enumerate() {
            osc.set_frequency(frequency * i as f32 + frequency)
        }
    }

    fn sync_reset(&mut self) {
        for ref mut osc in self.osc_s.iter_mut() {
            osc.sync_reset()
        }
    }
}
