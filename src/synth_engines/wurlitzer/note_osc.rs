use crate::{
    synth_engines::{
        synth::{build_sine_table, osc::WavetableOscillator},
        synth_common::{env::ADSR, lfo::LFO},
    },
    SampleGen,
};
use midi_control::MidiNote;

const KEY_FRAME_MOD: f32 = 0.68;
const FORMANT_SHIFTING: f32 = 0.11;

#[derive(Debug, Clone)]
pub struct ApLowPass {}

#[derive(Debug, Clone)]
pub struct ApHighPass {}

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
            f.wave_table = build_sine_table(&[2.0]);

            f
        };
        let trem_lfo = {
            let mut lfo = LFO::new();
            lfo.set_frequency(5.5);

            lfo
        };
        let vol_env = {
            let mut env = ADSR::new();
            env.set_atk(0.01);
            env.set_sus(0.125);
            env.set_decay(10.0);

            env
        };
        let param_env = {
            let mut env = ADSR::new();
            env.set_atk(0.01);
            env.set_sus(0.125);
            env.set_decay(7.0);

            env
        };
        let formant = {
            let mut f = WavetableOscillator::new();
            f.wave_table = build_sine_table(&[0.5]);

            f
        };

        Self {
            playing: None,
            osc_1,
            osc_2,
            trem_lfo,
            trem_lfo_depth: 0.25,
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

    pub fn release(&mut self) {
        self.vol_env.release();
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
        let mix = p_env - p_env * KEY_FRAME_MOD * self.vel;
        let trem_lfo = self.trem_lfo.get_sample() * self.trem_lfo_depth * self.vel;
        let vol = self.vol_env.get_samnple();

        if vol == 0.0 {
            self.playing = None;
        }

        harmonic_1 *= mix;
        fundimental *= 1.0 - mix;

        let mut sample = (fundimental + harmonic_1) * 0.5;

        // apply formant
        sample *= 1.0 - (self.formant.get_sample() * FORMANT_SHIFTING * p_env * self.vel);
        // TODO: apply COMB SPREAD FLANGE-

        sample *= vol;
        sample -= sample * trem_lfo;

        sample
    }
}
