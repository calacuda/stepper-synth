use crate::{
    effects::{Effect, EffectType},
    logger_init,
    sequencer::{Sequence, SequenceChannel, SequencerIntake, Step},
    synth_engines::{wave_table::WaveTableEngine, Synth, SynthEngine, SynthModule},
    HashMap, KnobCtrl, SampleGen, SAMPLE_RATE,
};
#[cfg(feature = "pyo3")]
use crate::{run_midi, sequencer::play_sequence};
use anyhow::{bail, Result};
use log::*;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::IndexMut,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};
use strum::EnumIter;
use tinyaudio::prelude::*;
use wavetable_synth::{
    common::{
        EnvParam, LfoParam, LowPass, LowPassParam, ModMatrixDest, ModMatrixItem, ModMatrixSrc,
        OscParam,
    },
    synth_engines::{
        synth::osc::OscTarget,
        synth_common::env::{ATTACK, DECAY, RELEASE, SUSTAIN},
    },
};

#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "stepper_synth_backend", get_all, eq, eq_int, hash, frozen)
)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub enum GuiParam {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "stepper_synth_backend", get_all, eq, eq_int, hash, frozen)
)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub enum Knob {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "stepper_synth_backend", get_all, eq, eq_int)
)]
#[derive(
    Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Hash, EnumIter, Serialize, Deserialize,
)]
pub enum SynthEngineType {
    SubSynth,
    B3Organ,
    Wurlitzer,
    WaveTable,
    // DrumSynth,
    // SamplerSynth,
}

impl Into<usize> for SynthEngineType {
    fn into(self) -> usize {
        match self {
            Self::B3Organ => 0,
            Self::SubSynth => 1,
            Self::Wurlitzer => 2,
            Self::WaveTable => 3,
        }
    }
}

impl Display for SynthEngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SubSynth => write!(f, "Subtract"),
            Self::B3Organ => write!(f, "Organ"),
            Self::Wurlitzer => write!(f, "Wurlitzer"),
            Self::WaveTable => write!(f, "WaveTable"),
            // Self::SamplerSynth => write!(f, "Sampler"),
        }
    }
}

#[cfg(feature = "pyo3")]
#[cfg_attr(feature = "pyo3", pymethods)]
impl SynthEngineType {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{self}"))
    }
}

#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
#[derive(Debug, Clone, Copy)]
pub enum Screen {
    Synth(SynthEngineType),
    Effect(EffectType),
    Stepper(i64),
    WaveTableSynth(),
    // Sequencer(),
    // MidiStepper(),
    // MidiSeq(),
}

#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SynthEngineState {
    pub engine: SynthEngineType,
    pub effect: EffectType,
    pub effect_on: bool,
    pub knob_params: HashMap<Knob, f32>,
    pub gui_params: HashMap<GuiParam, f32>,
}

#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscState {
    volume: f32,
    wave_table: Vec<f32>,
    on: bool,
    detune: f32,
    offset: i8,
    target: String,
}

impl From<WaveTableEngine> for Vec<OscState> {
    fn from(value: WaveTableEngine) -> Self {
        value.synth.voices[0]
            .lock()
            .unwrap()
            .oscs
            .clone()
            .map(|(osc, on)| OscState {
                volume: osc.level,
                wave_table: osc.wave_table.to_vec(),
                on,
                detune: osc.detune,
                offset: osc.offset as i8,
                target: match osc.target {
                    OscTarget::Filter1 => "Filter 1".into(),
                    OscTarget::Filter2 => "Filter 2".into(),
                    OscTarget::Filter1_2 => "Filter 1 & 2".into(),
                    OscTarget::Effects => "Effects".into(),
                    OscTarget::DirectOut => "Direct Out".into(),
                },
            })
            .to_vec()
    }
}

#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowPassState {
    cutoff: f32,
    res: f32,
    keytracking: bool,
    mix: f32,
}

impl From<WaveTableEngine> for Vec<LowPassState> {
    fn from(value: WaveTableEngine) -> Self {
        value.synth.voices[0]
            .lock()
            .unwrap()
            .filters
            .clone()
            .map(|lp| LowPassState {
                cutoff: lp.cutoff,
                res: lp.resonance,
                keytracking: lp.key_track,
                mix: lp.mix,
            })
            .to_vec()
    }
}

#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ADSRState {
    atk: f32,
    dcy: f32,
    sus: f32,
    rel: f32,
}

impl From<WaveTableEngine> for Vec<ADSRState> {
    fn from(value: WaveTableEngine) -> Self {
        value.synth.voices[0]
            .lock()
            .unwrap()
            .envs
            .clone()
            .map(|env| ADSRState {
                atk: env.base_params[ATTACK],
                dcy: env.base_params[DECAY],
                sus: env.base_params[SUSTAIN],
                rel: env.base_params[RELEASE],
            })
            .to_vec()
    }
}

#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LfoState {
    speed: f32,
}

impl From<WaveTableEngine> for Vec<LfoState> {
    fn from(value: WaveTableEngine) -> Self {
        value.synth.voices[0]
            .lock()
            .unwrap()
            .lfos
            .clone()
            .map(|lfo| LfoState {
                speed: 1.0 / lfo.freq,
            })
            .to_vec()
    }
}

#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModMatrixDisplayItem {
    pub src: String,
    pub dest: String,
    pub amt: f32,
    pub bipolar: bool,
    pub id: usize,
}

impl From<WaveTableEngine> for Vec<ModMatrixDisplayItem> {
    fn from(value: WaveTableEngine) -> Self {
        value
            .synth
            .mod_matrix
            .iter()
            .enumerate()
            .filter_map(|(i, mod_m)| {
                if let Some(entry) = mod_m {
                    let src = display_src(entry.src);
                    let dest = display_dest(entry.dest);

                    Some(ModMatrixDisplayItem {
                        src,
                        dest,
                        amt: entry.amt,
                        bipolar: entry.bipolar,
                        id: i,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

fn display_src(src: ModMatrixSrc) -> String {
    match src {
        ModMatrixSrc::Gate => "Gate".into(),
        ModMatrixSrc::Macro1 => "Macro-1".into(),
        ModMatrixSrc::Macro2 => "Macro-2".into(),
        ModMatrixSrc::Macro3 => "Macro-3".into(),
        ModMatrixSrc::Macro4 => "Macro-4".into(),
        ModMatrixSrc::Velocity => "Vel".into(),
        ModMatrixSrc::ModWheel => "Mod-Whl".into(),
        ModMatrixSrc::PitchWheel => "Pitch-Whl".into(),
        ModMatrixSrc::Env(n) => format!("Env-{n}"),
        ModMatrixSrc::Lfo(n) => format!("LFO-{n}"),
    }
}

fn display_dest(dest: ModMatrixDest) -> String {
    let display_lp_param = |param: LowPassParam| -> String {
        match param {
            LowPassParam::Cutoff => "CutOff".into(),
            LowPassParam::Res => "Res".into(),
            LowPassParam::Mix => "Mix".into(),
        }
    };

    match dest {
        ModMatrixDest::SynthVolume => "Vol.".into(),
        ModMatrixDest::ModMatrixEntryModAmt(id) => format!("Mod-Entry {id}"),
        ModMatrixDest::Osc {
            osc,
            param: OscParam::Level,
        } => format!("OSC {} Vol.", osc + 1),
        ModMatrixDest::Osc {
            osc,
            param: OscParam::Tune,
        } => format!("OSC {} Tune", osc + 1),
        ModMatrixDest::LowPass {
            low_pass: LowPass::LP1,
            param,
        } => format!("LowP 1 {}", display_lp_param(param)),
        ModMatrixDest::LowPass {
            low_pass: LowPass::LP2,
            param,
        } => format!("LowP 2 {}", display_lp_param(param)),
        ModMatrixDest::Env { env, param } => format!(
            "Env {env} {}",
            match param {
                EnvParam::Atk => "Atk",
                EnvParam::Dcy => "Dcy",
                EnvParam::Sus => "Sus",
                EnvParam::Rel => "Rel",
            }
        ),
        ModMatrixDest::Lfo {
            lfo,
            param: LfoParam::Speed,
        } => format!("LFO {lfo} Speed"),
    }
}

#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepperSynthState {
    Synth {
        engine: SynthEngineType,
        effect: EffectType,
        effect_on: bool,
        knob_params: HashMap<Knob, f32>,
        gui_params: HashMap<GuiParam, f32>,
    },
    Effect {
        effect: EffectType,
        effect_on: bool,
        params: HashMap<String, f32>,
    },
    MidiStepper {
        name: String,
        playing: bool,
        recording: bool,
        cursor: usize,
        tempo: u16,
        step: Step,
        sequence: Sequence,
        seq_n: usize,
        channel: SequenceChannel,
    },
    WaveTable {
        osc: Vec<OscState>,
        filter: Vec<LowPassState>,
        adsr: Vec<ADSRState>,
        lfo: Vec<LfoState>,
        mod_matrix: Vec<ModMatrixDisplayItem>,
    },
    // MidiSeq(),
}

#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WTSynthParam {
    OscVol {
        n: usize,
        to: f32,
    },
    OscWaveTable {
        n: usize,
        wave_table: Vec<f32>,
    },
    OscOn {
        n: usize,
        on: bool,
    },
    OscDetune {
        n: usize,
        detune: f32,
    },
    OscOffset {
        n: usize,
        offset: i16,
    },
    OscTarget {
        n: usize,
        target: i8,
    },
    LowPassCutoff {
        n: usize,
        cutoff: f32,
    },
    LowPassRes {
        n: usize,
        res: f32,
    },
    LowPassTracking {
        n: usize,
        track: bool,
    },
    LowPassMix {
        n: usize,
        mix: f32,
    },
    ADSRAttack {
        n: usize,
        val: f32,
    },
    ADSRDecay {
        n: usize,
        val: f32,
    },
    ADSRSustain {
        n: usize,
        val: f32,
    },
    ADSRRelease {
        n: usize,
        val: f32,
    },
    LfoSpeed {
        n: usize,
        speed: f32,
    },
    ModMatrixAdd {
        src: String,
        dest: String,
        amt: f32,
        bipolar: bool,
    },
    ModMatrixDel {
        /// the id of the mod matrix entry to rm (i.e its index + 1)
        id: usize,
    },
    ModMatrixMod {
        /// the id of the mod matrix entry to modify (i.e its index + 1)
        id: usize,
        src: String,
        dest: String,
        amt: f32,
        bipolar: bool,
    },
    // MidiLearn {},
    // MidiUnearn {},
}

#[cfg(feature = "pyo3")]
#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend"))]
#[derive(Debug)]
pub struct StepperSynth {
    // synth: Arc<Mutex<Synth>>,
    updated: Arc<Mutex<bool>>,
    screen: Screen,
    _handle: JoinHandle<()>,
    _midi_thread: JoinHandle<()>,
    exit: Arc<AtomicBool>,
    pub midi_sequencer: Arc<Mutex<SequencerIntake>>,
}

#[cfg(feature = "pyo3")]
impl StepperSynth {
    pub fn new() -> Self {
        // build synth in arc mutex
        let synth = Synth::new();
        let sequencer = Arc::new(Mutex::new(SequencerIntake::new(synth)));

        let updated = Arc::new(Mutex::new(true));
        let exit = Arc::new(AtomicBool::new(false));
        // let effect_midi = Arc::new(AtomicBool::new(false));

        let handle = {
            let seq = sequencer.clone();
            // let synth = synth.clone();
            let updated = updated.clone();
            let exit = exit.clone();
            // let effect_midi = effect_midi.clone();

            spawn(move || {
                let params = OutputDeviceParameters {
                    channels_count: 1,
                    sample_rate: SAMPLE_RATE as usize,
                    // channel_sample_count: 2048,
                    channel_sample_count: 1024,
                };
                let device = run_output_device(params, {
                    let seq = seq.clone();

                    move |data| {
                        for samples in data.chunks_mut(params.channels_count) {
                            let value = seq.lock().expect("couldn't lock synth").synth.get_sample();

                            for sample in samples {
                                *sample = value;
                            }
                        }
                    }
                });

                if let Err(e) = device {
                    error!("starting audio playback caused error: {e}");
                }

                // let seq = sequencer.clone();

                if let Err(e) = run_midi(seq, updated, exit) {
                    error!("{e}");
                }
            })
        };

        let thread = {
            let seq = sequencer.clone();

            spawn(move || loop {
                while !seq
                    .clone()
                    .lock()
                    .unwrap()
                    .state
                    .playing
                    .load(Ordering::Relaxed)
                {
                    sleep(Duration::from_secs_f64(0.001));
                }

                play_sequence(seq.clone());
            })
        };

        if let Err(reason) = logger_init() {
            eprintln!("failed to initiate logger because {reason}");
        }

        info!("Synth is ready to make sound");

        Self {
            // synth,
            updated,
            screen: Screen::Synth(SynthEngineType::B3Organ),
            _handle: handle,
            _midi_thread: thread,
            midi_sequencer: sequencer,
            exit,
            // effect_midi,
        }
    }

    pub fn get_engine_state(&self) -> SynthEngineState {
        let mut seq = self.midi_sequencer.lock().unwrap();

        SynthEngineState {
            engine: seq.synth.engine_type,
            effect: seq.synth.effect_type,
            effect_on: seq.synth.effect_power,
            knob_params: seq.synth.get_engine().get_params(),
            gui_params: seq.synth.get_engine().get_gui_params(),
        }
    }
}

#[cfg(feature = "pyo3")]
#[cfg_attr(feature = "pyo3", pymethods)]
impl StepperSynth {
    #[cfg(feature = "pyo3")]
    #[new]
    pub fn new_py() -> Self {
        Self::new()
    }

    pub fn exit(&mut self) {
        warn!("GoodBye");
        self.exit.store(true, Ordering::Relaxed);
    }

    pub fn updated(&self) -> bool {
        *self.updated.lock().unwrap()
    }

    pub fn toggle_effect_power(&mut self) {
        {
            let mut seq = self.midi_sequencer.lock().unwrap();
            seq.synth.effect_power = !seq.synth.effect_power;
        }
        self.set_updated();
    }

    fn set_updated(&mut self) {
        (*self.updated.lock().unwrap()) = true;
    }

    pub fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
        // info!("screen set");

        match screen {
            Screen::Effect(effect) => {
                self.set_effect(effect);
                // self.effect_midi.store(true, Ordering::Relaxed)
                self.midi_sequencer.lock().unwrap().synth.target_effects = true;
            }
            Screen::Synth(engine) => {
                self.set_engine(engine);
                // self.effect_midi.store(false, Ordering::Relaxed)
                self.midi_sequencer.lock().unwrap().synth.target_effects = false;
            }
            Screen::Stepper(seq) => {
                self.midi_sequencer.lock().unwrap().set_rec_head_seq(seq);
            } // Screen::Sequencer() => {}
            Screen::WaveTableSynth() => {
                self.set_engine(SynthEngineType::WaveTable);
                // self.effect_midi.store(false, Ordering::Relaxed)
                self.midi_sequencer.lock().unwrap().synth.target_effects = false;
            }
        }

        // info!("screen engine/effect set");

        self.set_updated();
    }

    pub fn get_screen(&self) -> Screen {
        self.screen
    }

    pub fn get_state(&self) -> Option<StepperSynthState> {
        // info!("get_state called");
        (*self.updated.lock().unwrap()) = false;
        // info!("after set");

        let mut seq = self.midi_sequencer.lock().unwrap();

        match self.screen {
            // Screen::Synth(SynthEngineType::B3Organ) => StepperSynthState::Synth {
            //     engine: SynthEngineType::B3Organ,
            //     effect: seq.synth.effect_type,
            //     effect_on: seq.synth.effect_power,
            //     knob_params: seq.synth.get_engine().get_params(),
            //     gui_params: seq.synth.get_engine().get_gui_params(),
            // },
            // Screen::Synth(SynthEngineType::SubSynth) => StepperSynthState::Synth {
            //     engine: SynthEngineType::SubSynth,
            //     effect: seq.synth.effect_type,
            //     effect_on: seq.synth.effect_power,
            //     knob_params: seq.synth.get_engine().get_params(),
            //     gui_params: seq.synth.get_engine().get_gui_params(),
            // },
            Screen::Synth(engine_type) => Some(StepperSynthState::Synth {
                engine: engine_type,
                effect: seq.synth.effect_type,
                effect_on: seq.synth.effect_power,
                knob_params: seq.synth.get_engine().get_params(),
                gui_params: seq.synth.get_engine().get_gui_params(),
            }),
            Screen::Effect(EffectType::Reverb) => Some(StepperSynthState::Effect {
                effect: EffectType::Reverb,
                effect_on: seq.synth.effect_power,
                params: seq.synth.get_effect().get_params(),
            }),
            Screen::Effect(EffectType::Chorus) => Some(StepperSynthState::Effect {
                effect: EffectType::Chorus,
                effect_on: seq.synth.effect_power,
                params: seq.synth.get_effect().get_params(),
            }),
            // Screen::Effect(EffectType::Delay) => StepperSynthState::Effect {
            //     effect: EffectType::Delay,
            //     effect_on: synth.effect_power,
            //     params: synth.effect.get_params(),
            // },
            Screen::Stepper(sequence) => {
                // if !seq.state.recording {
                // seq.rec_head.set
                // }
                seq.set_sequence(sequence.abs() as usize);

                Some(StepperSynthState::MidiStepper {
                    playing: seq.state.playing.load(Ordering::Relaxed),
                    recording: seq.state.recording,
                    name: seq.get_name(),
                    tempo: seq.bpm,
                    step: seq.get_step(false),
                    cursor: seq.get_cursor(false),
                    sequence: seq.get_sequence(),
                    seq_n: seq.rw_head.get_sequence(),
                    channel: seq.rw_head.get_channel(),
                })
            }
            Screen::WaveTableSynth() => {
                let synth = seq.synth.get_engine();

                let SynthModule::WaveTable(wt) = synth else {
                    return None;
                };

                let osc: Vec<OscState> = Vec::from(wt.clone());
                let adsr: Vec<ADSRState> = Vec::from(wt.clone());
                let filter: Vec<LowPassState> = Vec::from(wt.clone());
                let lfo: Vec<LfoState> = Vec::from(wt.clone());
                let mod_matrix: Vec<ModMatrixDisplayItem> = Vec::from(wt.clone());

                Some(StepperSynthState::WaveTable {
                    osc,
                    adsr,
                    filter,
                    lfo,
                    mod_matrix,
                })
            }
        }
    }

    pub fn set_engine(&mut self, engine: SynthEngineType) {
        if self.midi_sequencer.lock().unwrap().synth.set_engine(engine) {
            self.set_updated();
        }
    }

    pub fn set_effect(&mut self, effect: EffectType) {
        if self.midi_sequencer.lock().unwrap().synth.set_effect(effect) {
            self.set_updated();
        }
    }

    pub fn set_gui_param(&mut self, param: GuiParam, value: f32) {
        self.set_updated();
        let mut seq = self.midi_sequencer.lock().unwrap();

        match param {
            GuiParam::A => seq.synth.get_engine().gui_param_1(value),
            GuiParam::B => seq.synth.get_engine().gui_param_2(value),
            GuiParam::C => seq.synth.get_engine().gui_param_3(value),
            GuiParam::D => seq.synth.get_engine().gui_param_4(value),
            GuiParam::E => seq.synth.get_engine().gui_param_5(value),
            GuiParam::F => seq.synth.get_engine().gui_param_6(value),
            GuiParam::G => seq.synth.get_engine().gui_param_7(value),
            GuiParam::H => seq.synth.get_engine().gui_param_8(value),
        };
    }

    pub fn set_knob_param(&mut self, param: Knob, value: f32) {
        self.set_updated();
        let mut seq = self.midi_sequencer.lock().unwrap();
        let synth = &mut seq.synth;

        match (param, self.screen) {
            (Knob::One, Screen::Synth(_)) => synth.get_engine().knob_1(value),
            (Knob::Two, Screen::Synth(_)) => synth.get_engine().knob_2(value),
            (Knob::Three, Screen::Synth(_)) => synth.get_engine().knob_3(value),
            (Knob::Four, Screen::Synth(_)) => synth.get_engine().knob_4(value),
            (Knob::Five, Screen::Synth(_)) => synth.get_engine().knob_5(value),
            (Knob::Six, Screen::Synth(_)) => synth.get_engine().knob_6(value),
            (Knob::Seven, Screen::Synth(_)) => synth.get_engine().knob_7(value),
            (Knob::Eight, Screen::Synth(_)) => synth.get_engine().knob_8(value),
            (Knob::One, Screen::Effect(_)) => synth.get_effect().knob_1(value),
            (Knob::Two, Screen::Effect(_)) => synth.get_effect().knob_2(value),
            (Knob::Three, Screen::Effect(_)) => synth.get_effect().knob_3(value),
            (Knob::Four, Screen::Effect(_)) => synth.get_effect().knob_4(value),
            (Knob::Five, Screen::Effect(_)) => synth.get_effect().knob_5(value),
            (Knob::Six, Screen::Effect(_)) => synth.get_effect().knob_6(value),
            (Knob::Seven, Screen::Effect(_)) => synth.get_effect().knob_7(value),
            (Knob::Eight, Screen::Effect(_)) => synth.get_effect().knob_8(value),
            (_, Screen::Stepper(_)) => false,
            _ => false,
        };
    }

    // /// increments the record head to the next step
    // pub fn next_step(&mut self) {}

    // /// sets the record head to a step
    // pub fn set_rec_head_step(&mut self, step: usize) {}

    pub fn start_recording(&mut self) {
        self.set_updated();

        // self.midi_sequencer
        //     .lock()
        //     .unwrap()
        //     .state
        //     .playing
        //     .store(false, Ordering::Relaxed);
        self.midi_sequencer.lock().unwrap().state.recording = true;
    }

    pub fn stop_seq(&mut self) {
        self.set_updated();

        self.midi_sequencer
            .lock()
            .unwrap()
            .state
            .playing
            .store(false, Ordering::Relaxed);
        self.midi_sequencer.lock().unwrap().state.recording = false;
    }

    pub fn start_playing(&mut self) {
        self.set_updated();

        self.midi_sequencer
            .lock()
            .unwrap()
            .state
            .playing
            .store(true, Ordering::Relaxed);
        self.midi_sequencer.lock().unwrap().state.recording = false;
    }

    pub fn prev_sequence(&mut self) {
        self.midi_sequencer.lock().unwrap().prev_sequence();
        self.set_updated();
    }

    pub fn next_sequence(&mut self) {
        self.midi_sequencer.lock().unwrap().next_sequence();
        self.set_updated()
    }

    pub fn next_step(&mut self) {
        self.midi_sequencer.lock().unwrap().next_step();
        self.set_updated()
    }

    pub fn prev_step(&mut self) {
        self.midi_sequencer.lock().unwrap().prev_step();
        self.set_updated()
    }

    pub fn tempo_up(&mut self) {
        self.set_updated();
        let mut seq = self.midi_sequencer.lock().unwrap();

        seq.bpm = (seq.bpm + 1) % u16::MAX;

        if seq.bpm == 0 {
            seq.bpm = 1;
        }
    }

    pub fn tempo_down(&mut self) {
        self.set_updated();
        let mut seq = self.midi_sequencer.lock().unwrap();

        if seq.bpm > 0 {
            seq.bpm = (seq.bpm - 1) % u16::MAX;
        }

        if seq.bpm == 0 {
            seq.bpm = 1;
        }
    }

    pub fn add_step(&mut self) {
        self.set_updated();
        let mut seq = self.midi_sequencer.lock().unwrap();
        seq.add_step();
    }

    pub fn del_step(&mut self) {
        self.set_updated();
        let mut seq = self.midi_sequencer.lock().unwrap();
        seq.del_step();
    }

    pub fn set_seq_channel(&mut self, channel: SequenceChannel) {
        self.set_updated();
        self.midi_sequencer.lock().unwrap().set_rec_channel(channel);
    }

    pub fn prev_channel(&mut self) {
        self.set_updated();
        self.midi_sequencer.lock().unwrap().rw_head.prev_channel();
    }

    pub fn next_channel(&mut self) {
        self.set_updated();
        self.midi_sequencer.lock().unwrap().rw_head.next_channel();
    }

    pub fn wt_param_setter(&mut self, param: WTSynthParam) {
        self.set_updated();
        let mut seq = self.midi_sequencer.lock().unwrap();
        let synth = &mut seq.synth;

        let SynthModule::WaveTable(wt_synth) = synth.get_engine() else {
            return;
        };

        match param {
            WTSynthParam::OscOn { n, on } => {
                wt_synth
                    .synth
                    .voices
                    .iter()
                    .for_each(|v| v.lock().unwrap().oscs[n].1 = on);
            }
            WTSynthParam::OscVol { n, to } => wt_synth
                .synth
                .voices
                .iter()
                .for_each(|v| v.lock().unwrap().oscs.index_mut(n).0.level = to),
            WTSynthParam::OscDetune { n, detune } => wt_synth
                .synth
                .voices
                .iter()
                .for_each(|v| v.lock().unwrap().oscs[n].0.detune = detune),
            WTSynthParam::OscWaveTable {
                n: _,
                wave_table: _,
            } => {
                // wt_synth
                //     .synth
                //     .voices
                //     .iter()
                //     .for_each(|v| v.lock().unwrap().oscs[n].0.wave_table = wave_table);
                // TODO: make happen
            }
            WTSynthParam::OscOffset { n, offset } => {
                wt_synth
                    .synth
                    .voices
                    .iter()
                    .for_each(|v| v.lock().unwrap().oscs[n].0.offset = offset);
            }
            WTSynthParam::OscTarget { n: _, target: _ } => {
                // wt_synth
                // .synth
                // .voices
                // .iter()
                // .for_each(|v| v.lock().unwrap().oscs[n].0.target += target);
                // TODO: make happen
            }
            WTSynthParam::LowPassCutoff { n, cutoff } => {
                wt_synth
                    .synth
                    .voices
                    .iter()
                    .for_each(|v| v.lock().unwrap().filters[n].cutoff = cutoff);
            }
            WTSynthParam::LowPassRes { n, res } => {
                wt_synth
                    .synth
                    .voices
                    .iter()
                    .for_each(|v| v.lock().unwrap().filters[n].resonance = res);
            }
            WTSynthParam::LowPassMix { n, mix } => {
                wt_synth
                    .synth
                    .voices
                    .iter()
                    .for_each(|v| v.lock().unwrap().filters[n].mix = mix);
            }
            WTSynthParam::LowPassTracking { n, track } => {
                wt_synth
                    .synth
                    .voices
                    .iter()
                    .for_each(|v| v.lock().unwrap().filters[n].key_track = track);
            }
            WTSynthParam::ADSRAttack { n, val } => {
                wt_synth
                    .synth
                    .voices
                    .iter()
                    .for_each(|v| v.lock().unwrap().envs[n].set_atk(val));
            }
            WTSynthParam::ADSRDecay { n, val } => {
                wt_synth
                    .synth
                    .voices
                    .iter()
                    .for_each(|v| v.lock().unwrap().envs[n].set_decay(val));
            }
            WTSynthParam::ADSRSustain { n, val } => {
                wt_synth
                    .synth
                    .voices
                    .iter()
                    .for_each(|v| v.lock().unwrap().envs[n].set_sus(val));
            }
            WTSynthParam::ADSRRelease { n, val } => {
                wt_synth
                    .synth
                    .voices
                    .iter()
                    .for_each(|v| v.lock().unwrap().envs[n].set_release(val));
            }
            WTSynthParam::LfoSpeed { n, speed } => {
                wt_synth
                    .synth
                    .voices
                    .iter()
                    .for_each(|v| v.lock().unwrap().lfos[n].set_frequency(1.0 / speed));
            }
            WTSynthParam::ModMatrixAdd {
                src,
                dest,
                amt,
                bipolar,
            } => {
                let Ok(src) = str_to_mod_src(&src) else {
                    error!("the source {src:?} failed to convert to ModMatrixSrc");
                    return;
                };
                // let Ok(dest) = str_to_mod_dest(&dest) else {
                //     error!("the destination {dest:?} failed to convert to ModMatrixDest");
                //     return;
                // };
                // let s = ModMatrixSrc::Velocity;
                // warn!("{:?}", toml::to_string_pretty(&s));

                // let Ok(src) = toml::from_str::<ModMatrixSrc>(&src) else {
                //     error!("the source {src:?} failed to convert to ModMatrixSrc");
                //     return;
                // };
                // info!("src => {src:?}");

                // let d = ModMatrixDest::SynthVolume;
                // warn!("{:?}", toml::to_string_pretty(&d));

                let dest = if dest.to_string().to_lowercase().starts_with("vol") {
                    ModMatrixDest::SynthVolume
                } else {
                    let Ok(dest) = toml::from_str::<ModMatrixDest>(&dest) else {
                        error!("the destination {dest:?} failed to convert to ModMatrixDest");
                        return;
                    };

                    dest
                };
                // info!("dest => {dest:?}");
                let matrix_item = ModMatrixItem {
                    src,
                    dest,
                    amt,
                    bipolar,
                };

                info!("adding matrix item {matrix_item:?} to the mod_matrix");

                for item in wt_synth.synth.mod_matrix.iter_mut() {
                    if item.is_none() {
                        *item = Some(matrix_item);
                        break;
                    }
                }
            }
            WTSynthParam::ModMatrixDel { id } => {
                let mut to_rm = [id].to_vec();

                loop {
                    let Some(id) = to_rm.pop() else {
                        break;
                    };

                    let matrix = wt_synth.synth.mod_matrix.clone();

                    // rm the identified matrix entry & scootch everything after it down.
                    for i in (id + 1)..matrix.len() {
                        wt_synth.synth.mod_matrix[i - 1] = matrix[i];
                    }

                    // if any matrix entries modulate the amount of a matrix entry with an id GREATER
                    // then the rm'ed id adjust to account for the shift from the above for loop.
                    //
                    // if any matrix entries modulate the amount of a matrix entry with an id EQUAL to
                    // that of the rm'ed id rm them too.
                    for (i, item) in wt_synth.synth.mod_matrix.iter_mut().enumerate() {
                        if let Some(ref mut entry) = item {
                            match entry.dest {
                                ModMatrixDest::ModMatrixEntryModAmt(ref mut n) => {
                                    if *n > id {
                                        *n -= 1
                                    } else if *n == id {
                                        // // recurse
                                        // self.wt_param_setter(WTSynthParam::ModMatrixDel { id: i })
                                        to_rm.push(i);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            WTSynthParam::ModMatrixMod {
                id,
                src,
                dest,
                amt,
                bipolar,
            } => {
                let Ok(src) = str_to_mod_src(&src) else {
                    error!("the source {src:?} failed to convert to ModMatrixSrc");
                    return;
                };
                // let Ok(dest) = str_to_mod_dest(&dest) else {
                //     error!("the destination {dest:?} failed to convert to ModMatrixDest");
                //     return;
                // };
                // let Ok(src) = toml::from_str::<ModMatrixSrc>(&src) else {
                //     error!("the source {src:?} failed to convert to ModMatrixSrc");
                //     return;
                // };
                let Ok(dest) = toml::from_str::<ModMatrixDest>(&dest) else {
                    error!("the destination {dest:?} failed to convert to ModMatrixDest");
                    return;
                };
                let matrix_item = ModMatrixItem {
                    src,
                    dest,
                    amt,
                    bipolar,
                };

                wt_synth.synth.mod_matrix[id - 1] = Some(matrix_item);
            }
            _ => {}
        }
    }
}

fn str_to_mod_src(src: &str) -> Result<ModMatrixSrc> {
    let src = src.trim().to_lowercase();

    if src.starts_with("env-") {
        let n: usize = src.split("-").collect::<Vec<_>>()[1].parse()?;

        return Ok(ModMatrixSrc::Env(n - 1));
    }

    if src.starts_with("lfo-") {
        let n: usize = src.split("-").collect::<Vec<_>>()[1].parse()?;

        return Ok(ModMatrixSrc::Lfo(n - 1));
    }

    Ok(match src.as_str() {
        "velocity" | "vel" => ModMatrixSrc::Velocity,
        "gate" => ModMatrixSrc::Gate,
        "mod-wheel" | "mod-whl" => ModMatrixSrc::ModWheel,
        "pitch-wheel" | "pitch-whl" => ModMatrixSrc::PitchWheel,
        "macro-1" | "macro1" | "m-1" | "m1" => ModMatrixSrc::Macro1,
        "macro-2" | "macro2" | "m-2" | "m2" => ModMatrixSrc::Macro1,
        "macro-3" | "macro3" | "m-3" | "m3" => ModMatrixSrc::Macro1,
        "macro-4" | "macro4" | "m-4" | "m4" => ModMatrixSrc::Macro1,
        // "" => ModMatrixSrc::,
        _ => bail!(""),
    })
}

// fn str_to_mod_dest(dest: &str) -> Result<ModMatrixDest> {
//
// }
