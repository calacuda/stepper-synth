use crate::{
    synth_engines::{
        synth::{build_sine_table, osc::WavetableOscillator},
        synth_common::{env::ADSR, lfo::LFO, WaveTable, WAVE_TABLE_SIZE},
    },
    SampleGen,
};
use midi_control::MidiNote;

const WURLI_ENV_ATTACK: f32 = 0.015;
const WURLI_ENV_SUS: f32 = 0.0625;
const WURLI_ENV_REL: f32 = 0.01;
const KEY_FRAME_MOD: f32 = 0.68;
const FORMANT_SHIFTING: f32 = 0.11;
const TREM_DEPTH_MOD_AMT: f32 = 0.585;

// #[derive(Debug, Clone)]
// pub struct ApLowPass {}
//
// #[derive(Debug, Clone)]
// pub struct ApHighPass {}

pub fn build_wurli_overtone_table(overtones: &[(f64, f64)]) -> WaveTable {
    let mut wave_table = [0.0; WAVE_TABLE_SIZE];

    let n_overtones = overtones.len();

    let bias = 1.0 / (n_overtones as f32 * 0.5);

    for i in 0..WAVE_TABLE_SIZE {
        for (ot, weight) in overtones {
            wave_table[i] +=
                ((2.0 * core::f64::consts::PI * i as f64 * ot / WAVE_TABLE_SIZE as f64).sin()
                    / (1.0 / weight)) as f32
        }

        wave_table[i] *= bias;
    }

    wave_table.into()
}

#[derive(Debug, Clone)]
pub struct WurliNoteOsc {
    pub playing: Option<MidiNote>,
    osc_1: WavetableOscillator,
    osc_2: WavetableOscillator,
    formant: WavetableOscillator,
    trem_lfo: LFO,
    pub trem_lfo_depth: f32,
    // comb_filter
    pub vol_env: ADSR,
    param_env: ADSR,
    base_frequency: f32,
    frequency: f32,
    vel: f32,
}

impl WurliNoteOsc {
    pub fn new() -> Self {
        let osc_1 = {
            let mut f = WavetableOscillator::new();
            f.wave_table = build_sine_table(&[1.0]);

            f
        };
        let osc_2 = {
            let mut f = WavetableOscillator::new();
            f.wave_table = build_wurli_overtone_table(&[
                (2.0, 1.0),
                (3.0, 0.5),
                (4.0, 0.25),
                (5.0, 0.25),
                (6.0, 0.125),
            ]);

            f
        };
        let trem_lfo = {
            let mut lfo = LFO::new();
            lfo.set_frequency(5.5);

            lfo
        };
        let vol_env = {
            let mut env = ADSR::new();
            env.set_atk(WURLI_ENV_ATTACK);
            env.set_sus(WURLI_ENV_SUS);
            env.set_decay(10.0);
            env.set_release(WURLI_ENV_REL);

            env
        };
        let param_env = {
            let mut env = ADSR::new();
            env.set_atk(WURLI_ENV_ATTACK);
            env.set_sus(WURLI_ENV_SUS);
            env.set_decay(7.0);
            env.set_release(WURLI_ENV_REL);

            env
        };
        let formant = {
            let mut f = WavetableOscillator::new();
            // f.wave_table = build_sine_table(&[0.5]);
            f.wave_table = build_wurli_overtone_table(&[
                (0.5, 1.0),
                (1.0, 1.0),
                // (2.0, 1.0),
                (3.0, 0.75),
                // (4.0, 0.25),
                (5.0, 0.5),
                // (6.0, 0.125),
            ]);

            f
        };

        Self {
            playing: None,
            osc_1,
            osc_2,
            trem_lfo,
            trem_lfo_depth: 0.5,
            vol_env,
            param_env,
            formant,
            base_frequency: 0.0,
            frequency: 0.0,
            vel: 0.0,
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.vol_env.pressed()
    }

    pub fn press(&mut self, midi_note: u8, vel: f32) {
        // info!("playing note: {midi_note} with vel: {vel}");
        self.vel = vel;
        self.vol_env.press();
        self.param_env.press();
        self.frequency = Self::get_freq(midi_note);
        self.base_frequency = self.frequency;

        self.osc_1.set_frequency(self.frequency);
        self.osc_2.set_frequency(self.frequency);
        // self.low_pass.set_note(self.frequency);
        self.playing = Some(midi_note);
    }

    fn get_freq(midi_note: u8) -> f32 {
        let exp = (f32::from(midi_note) + 36.376_316) / 12.0;
        // 2_f32.powf(exp)

        2.0_f32.powf(exp)
    }

    pub fn set_trem_depth(&mut self, depth: f32) {
        self.trem_lfo_depth = depth;
    }

    pub fn release(&mut self) {
        self.vol_env.release();
        self.param_env.release();
        // self.playing = None;
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
        self.osc_1.set_frequency(new_freq);
        self.osc_2.set_frequency(new_freq);
        // println!("frequency => {}", self.frequency);
        // println!("new_freq => {}", new_freq);
        self.frequency = new_freq;
    }

    pub fn unbend(&mut self) {
        // println!("unbend => {}", self.base_frequency);
        self.osc_1.set_frequency(self.base_frequency);
        self.osc_2.set_frequency(self.base_frequency);
        self.frequency = self.base_frequency;
    }
}

impl SampleGen for WurliNoteOsc {
    fn get_sample(&mut self) -> f32 {
        let mut harmonic_1 = self.osc_2.get_sample();
        let mut fundimental = self.osc_1.get_sample();
        let p_env = self.param_env.get_samnple();
        let mix = p_env * KEY_FRAME_MOD * self.vel;
        // let mix = p_env * KEY_FRAME_MOD * self.vel;
        let trem_lfo = self.trem_lfo.get_sample() * self.trem_lfo_depth * TREM_DEPTH_MOD_AMT;
        let mut vol = self.vol_env.get_samnple();

        if vol == 0.0 {
            self.playing = None;
        } else {
            vol *= self.vel;
        }

        harmonic_1 *= 0.5 + mix;
        fundimental *= 0.5 - mix;
        // info!(
        //     "harmonic 1 vol: {mix}, fundimental vol: {}, total vol: {}",
        //     1.0 - mix,
        //     mix + (1.0 - mix)
        // );

        let formant = self.formant.get_sample() * FORMANT_SHIFTING * p_env * self.vel;

        let sample = (fundimental + harmonic_1
            // * (formant * FORMANT_SHIFTING * p_env * self.vel)
            
            + (formant * (1.0 - FORMANT_SHIFTING * p_env * self.vel))) * 0.5 * vol;

        // apply formant
        // sample *= 1.0 - (self.formant.get_sample() * FORMANT_SHIFTING * p_env * self.vel);
        // TODO: apply COMB SPREAD FLANGE-

        // sample *= vol;
        sample + sample * ((trem_lfo - (TREM_DEPTH_MOD_AMT * 0.5)) * self.vel)
        // sample
    }
}
