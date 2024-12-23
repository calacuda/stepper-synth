use crate::{
    effects::EffectType, logger_init, run_midi, synth_engines::Synth, HashMap, KnobCtrl, SampleGen,
    SAMPLE_RATE,
};
use log::*;
use pyo3::prelude::*;
use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{spawn, JoinHandle},
};
use tinyaudio::prelude::*;

#[pyclass(module = "stepper_synth_backend", get_all, eq, eq_int, hash, frozen)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
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

#[pyclass(module = "stepper_synth_backend", get_all, eq, eq_int, hash, frozen)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
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

#[pyclass(module = "stepper_synth_backend", get_all, eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum SynthEngineType {
    SubSynth,
    B3Organ,
    // DrumSynth,
    // SamplerSynth,
    // WaveTableSynth,
}

impl Display for SynthEngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SubSynth => write!(f, "Subtract"),
            Self::B3Organ => write!(f, "Organ"),
            // Self::SamplerSynth => write!(f, "Sampler"),
            // Self::WaveTableSynth => write!(f, "WaveTbl"),
        }
    }
}

#[pymethods]
impl SynthEngineType {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{self}"))
    }
}

// #[pyclass(module = "stepper_synth_backend", get_all)]
// #[derive(Debug, Clone, PartialEq, PartialOrd)]
// pub enum PythonCmd {
//     // SetSynthParam(SynthParam),
//     SetGuiParam { param: GuiParam, set_to: f32 },
//     SetKnob { knob: Knob, set_to: f32 },
//     ChangeSynthEngine(SynthEngineType),
//     EffectPowerToggle(),
//     ChangeEffectType(EffectType),
//     Exit(),
// }

// pub type State = SynthParam;

// #[pyclass(module = "stepper_synth_backend", get_all)]
// #[derive(Debug, Clone)]
// pub struct State {
//     pub engine: SynthEngineType,
//     pub effect: EffectType,
//     pub effect_on: bool,
//     pub knob_params: HashMap<Knob, f32>,
//     pub gui_params: HashMap<GuiParam, f32>,
// }

#[pyclass(module = "stepper_synth_backend", get_all)]
#[derive(Debug, Clone, Copy)]
pub enum Screen {
    Synth(SynthEngineType),
    Effect(EffectType),
    // MidiStepper(),
    // MidiSeq(),
}

#[pyclass(module = "stepper_synth_backend", get_all)]
#[derive(Debug, Clone)]
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
    // MidiStepper(),
    // MidiSeq(),
}

#[pyclass(module = "stepper_synth_backend")]
#[derive(Debug)]
pub struct StepperSynth {
    synth: Arc<Mutex<Synth>>,
    updated: Arc<Mutex<bool>>,
    screen: Screen,
    _handle: JoinHandle<()>,
    exit: Arc<AtomicBool>,
    effect_midi: Arc<AtomicBool>,
}

#[pymethods]
impl StepperSynth {
    #[new]
    pub fn new() -> Self {
        // build synth in arc mutex
        let synth = Arc::new(Mutex::new(Synth::new()));

        let updated = Arc::new(Mutex::new(true));
        let exit = Arc::new(AtomicBool::new(false));
        let effect_midi = Arc::new(AtomicBool::new(false));

        let handle = {
            let s = synth.clone();
            let u = updated.clone();
            let exit = exit.clone();
            let effect_midi = effect_midi.clone();

            spawn(move || {
                // let res = ;

                // if let Err(reason) = logger_init() {
                //     eprintln!("failed to initiate logger because {reason}");
                // } else {
                //     debug!("logger initiated");
                // }

                let params = OutputDeviceParameters {
                    channels_count: 1,
                    sample_rate: SAMPLE_RATE as usize,
                    // channel_sample_count: 2048,
                    channel_sample_count: 1024,
                };
                let device = {
                    let s = s.clone();

                    run_output_device(params, move |data| {
                        for samples in data.chunks_mut(params.channels_count) {
                            let value = s.lock().expect("couldn't lock synth").get_sample();

                            for sample in samples {
                                *sample = value;
                            }
                        }
                    })
                };

                if let Err(e) = device {
                    error!("strating audio playback caused error: {e}");
                }

                if let Err(e) = run_midi(s, u, exit, effect_midi) {
                    error!("{e}");
                }
            })
        };

        if let Err(reason) = logger_init() {
            eprintln!("failed to initiate logger because {reason}");
        }

        info!("Synth is ready to make sound");

        Self {
            synth,
            updated,
            screen: Screen::Synth(SynthEngineType::B3Organ),
            _handle: handle,
            exit,
            effect_midi,
        }
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
            let mut synth = self.synth.lock().unwrap();
            synth.effect_power = !synth.effect_power;
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
                self.effect_midi.store(true, Ordering::Relaxed)
            }
            Screen::Synth(engine) => {
                self.set_engine(engine);
                self.effect_midi.store(false, Ordering::Relaxed)
            }
        }

        // info!("screen engine/effect set");

        self.set_updated();
    }

    pub fn get_state(&self) -> StepperSynthState {
        // info!("get_state called");
        (*self.updated.lock().unwrap()) = false;
        // info!("after set");

        let synth = self.synth.lock().unwrap();

        match self.screen {
            Screen::Synth(SynthEngineType::B3Organ) => StepperSynthState::Synth {
                engine: SynthEngineType::B3Organ,
                effect: synth.effect.effect,
                effect_on: synth.effect_power,
                knob_params: synth.engine.get_params(),
                gui_params: synth.engine.get_gui_params(),
            },
            Screen::Synth(SynthEngineType::SubSynth) => StepperSynthState::Synth {
                engine: SynthEngineType::SubSynth,
                effect: synth.effect.effect,
                effect_on: synth.effect_power,
                knob_params: synth.engine.get_params(),
                gui_params: synth.engine.get_gui_params(),
            },
            Screen::Effect(EffectType::Reverb) => StepperSynthState::Effect {
                effect: EffectType::Reverb,
                effect_on: synth.effect_power,
                params: synth.effect.get_params(),
            },
            Screen::Effect(EffectType::Chorus) => StepperSynthState::Effect {
                effect: EffectType::Chorus,
                effect_on: synth.effect_power,
                params: synth.effect.get_params(),
            },
            Screen::Effect(EffectType::Delay) => StepperSynthState::Effect {
                effect: EffectType::Delay,
                effect_on: synth.effect_power,
                params: synth.effect.get_params(),
            },
        }
    }

    pub fn set_engine(&mut self, engine: SynthEngineType) {
        if self.synth.lock().unwrap().set_engine(engine) {
            self.set_updated();
        }
    }

    pub fn set_effect(&mut self, effect: EffectType) {
        if self.synth.lock().unwrap().set_effect(effect) {
            // (*self.updated.lock().unwrap()) = true;
            self.set_updated();
        }
    }

    pub fn set_gui_param(&mut self, param: GuiParam, value: f32) {
        self.set_updated();
        let mut synth = self.synth.lock().unwrap();

        match param {
            GuiParam::A => synth.engine.gui_param_1(value),
            GuiParam::B => synth.engine.gui_param_2(value),
            GuiParam::C => synth.engine.gui_param_3(value),
            GuiParam::D => synth.engine.gui_param_4(value),
            GuiParam::E => synth.engine.gui_param_5(value),
            GuiParam::F => synth.engine.gui_param_6(value),
            GuiParam::G => synth.engine.gui_param_7(value),
            GuiParam::H => synth.engine.gui_param_8(value),
        };
    }

    pub fn set_knob_param(&mut self, param: Knob, value: f32) {
        self.set_updated();
        let mut synth = self.synth.lock().unwrap();

        match (param, self.screen) {
            (Knob::One, Screen::Synth(_)) => synth.engine.knob_1(value),
            (Knob::Two, Screen::Synth(_)) => synth.engine.knob_2(value),
            (Knob::Three, Screen::Synth(_)) => synth.engine.knob_3(value),
            (Knob::Four, Screen::Synth(_)) => synth.engine.knob_4(value),
            (Knob::Five, Screen::Synth(_)) => synth.engine.knob_5(value),
            (Knob::Six, Screen::Synth(_)) => synth.engine.knob_6(value),
            (Knob::Seven, Screen::Synth(_)) => synth.engine.knob_7(value),
            (Knob::Eight, Screen::Synth(_)) => synth.engine.knob_8(value),
            (Knob::One, Screen::Effect(_)) => synth.effect.knob_1(value),
            (Knob::Two, Screen::Effect(_)) => synth.effect.knob_2(value),
            (Knob::Three, Screen::Effect(_)) => synth.effect.knob_3(value),
            (Knob::Four, Screen::Effect(_)) => synth.effect.knob_4(value),
            (Knob::Five, Screen::Effect(_)) => synth.effect.knob_5(value),
            (Knob::Six, Screen::Effect(_)) => synth.effect.knob_6(value),
            (Knob::Seven, Screen::Effect(_)) => synth.effect.knob_7(value),
            (Knob::Eight, Screen::Effect(_)) => synth.effect.knob_8(value),
        };
    }
}
