#![feature(let_chains, stmt_expr_attributes)]
// #![feature(let_chains)]
// use anyhow::Result;
use effects::reverb::ReverbParam;
use effects::EffectType;
use effects::EffectsModule;
use enum_dispatch::enum_dispatch;
use fern::colors::{Color, ColoredLevelConfig};
use fxhash::FxHashMap;
use fxhash::FxHashSet;
use log::*;
use midi_control::MidiMessage;
#[cfg(feature = "midir")]
use midir::MidiInput;
#[cfg(feature = "midir")]
use midir::{Ignore, PortInfoError};
use pygame_coms::Screen;
#[cfg(feature = "pyo3")]
use pygame_coms::StepperSynth;
use pygame_coms::StepperSynthState;
use pygame_coms::{GuiParam, Knob, SynthEngineType};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use sequencer::Sequence;
use sequencer::SequencerIntake;
use sequencer::Step;
use sequencer::StepCmd;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use synth_engines::synth::OscType;
use synth_engines::LfoInput;
use synth_engines::Param;
use synth_engines::Synth;
use synth_engines::SynthModule;

pub type HashMap<Key, Val> = FxHashMap<Key, Val>;
pub type HashSet<T> = FxHashSet<T>;

pub const SAMPLE_RATE: u32 = 48_000;
pub const CHANNEL_SIZE: usize = 1_024;

pub mod effects;
pub mod pygame_coms;
pub mod sequencer;
pub mod synth_engines;

pub trait MidiControlled {
    fn midi_input(&mut self, message: &MidiMessage);
}

#[enum_dispatch(EffectsModule, SynthModule)]
pub trait SampleGen {
    fn get_sample(&mut self) -> f32;
}

#[allow(unused_variables)]
#[enum_dispatch(EffectsModule, SynthModule)]
pub trait KnobCtrl {
    // parameters edited by the MIDI controllers built in knobs
    fn knob_1(&mut self, value: f32) -> bool {
        false
    }
    fn knob_2(&mut self, value: f32) -> bool {
        false
    }
    fn knob_3(&mut self, value: f32) -> bool {
        false
    }
    fn knob_4(&mut self, value: f32) -> bool {
        false
    }
    fn knob_5(&mut self, value: f32) -> bool {
        false
    }
    fn knob_6(&mut self, value: f32) -> bool {
        false
    }
    fn knob_7(&mut self, value: f32) -> bool {
        false
    }
    fn knob_8(&mut self, value: f32) -> bool {
        false
    }

    // the parameters edited by the GUI
    fn gui_param_1(&mut self, value: f32) -> bool {
        false
    }
    fn gui_param_2(&mut self, value: f32) -> bool {
        false
    }
    fn gui_param_3(&mut self, value: f32) -> bool {
        false
    }
    fn gui_param_4(&mut self, value: f32) -> bool {
        false
    }
    fn gui_param_5(&mut self, value: f32) -> bool {
        false
    }
    fn gui_param_6(&mut self, value: f32) -> bool {
        false
    }
    fn gui_param_7(&mut self, value: f32) -> bool {
        false
    }
    fn gui_param_8(&mut self, value: f32) -> bool {
        false
    }

    fn get_lfo_input(&mut self) -> &mut LfoInput;

    fn lfo_connect(&mut self, param: Param) {
        self.get_lfo_input().target = Some(param);
    }

    fn lfo_disconnect(&mut self) {
        self.get_lfo_input().target = None;
    }

    fn lfo_control(&mut self, lfo_sample: f32) {
        self.get_lfo_input().sample = lfo_sample;
    }
}

pub struct Player {
    pub synth: Arc<Mutex<Synth>>,
}

impl Iterator for Player {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.synth.lock().expect("couldn't lock synth").get_sample();
        Some(sample)
    }
}

#[cfg(feature = "pyo3")]
fn run_midi(
    synth: Arc<Mutex<SequencerIntake>>,
    updated: Arc<Mutex<bool>>,
    exit: Arc<AtomicBool>,
    // effect_midi: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    let mut registered_ports = HashMap::default();

    while !exit.load(Ordering::Relaxed) {
        let mut midi_in = MidiInput::new("midir reading input")?;
        midi_in.ignore(Ignore::None);

        // Get an input port (read from console if multiple are available)
        let in_ports = midi_in.ports();
        let port_names: Vec<std::result::Result<String, PortInfoError>> = in_ports
            .iter()
            .map(|port| midi_in.port_name(port))
            .collect();
        registered_ports.retain(|k: &String, _| port_names.contains(&Ok(k.clone())));

        for in_port in in_ports.iter() {
            let Ok(port_name) = midi_in.port_name(in_port) else {
                continue;
            };

            if registered_ports.contains_key(&port_name) {
                continue;
            }

            info!("port {port_name}");
            let mut midi_in = MidiInput::new("midir reading input")?;
            midi_in.ignore(Ignore::None);
            // let synth = synth.clone();
            // let tx = tx.clone();
            let updated = updated.clone();
            // let effect = effect_midi.clone();

            registered_ports.insert(
                port_name,
                midi_in.connect(
                    in_port,
                    "midir-read-input",
                    {
                        let synth = synth.clone();
                        move |_stamp, message, _| {
                            let message = MidiMessage::from(message);
                            let send = || {
                                let mut u = updated.lock().unwrap();
                                *u = true;
                            };

                            // do midi stuff
                            synth.lock().unwrap().midi_input(&message);
                            send();
                        }
                    },
                    (),
                ),
            );
        }
    }

    Ok(())
}

fn logger_init() -> Result<()> {
    let colors = ColoredLevelConfig::new()
        .debug(Color::Blue)
        .info(Color::Green)
        .warn(Color::Magenta)
        .error(Color::Red);

    #[cfg(debug_assertions)]
    let dis = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .chain(std::io::stderr());
    // .filter(|metadata| metadata..starts_with("stepper"))
    // .chain(fern::log_file("stepper-synth.log")?);
    // .apply()?;

    #[cfg(not(debug_assertions))]
    let dis = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] {}",
                colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stderr());
    // .chain(fern::log_file("stepper-synth.log")?);
    // .filter(|metadata| metadata.target().starts_with("stepper"))
    // .apply()?;

    #[cfg(target_arch = "aarch64")]
    let dis = dis.chain(fern::log_file("stepper-synth.log")?);

    dis.apply()?;

    info!("logger started");

    Ok(())
}

#[cfg(feature = "pyo3")]
#[pyfunction]
fn log_trace(msg: String) {
    // println!("{msg}");
    trace!("{msg}")
}

#[cfg(feature = "pyo3")]
#[pyfunction]
fn log_debug(msg: String) {
    debug!("{msg}")
}

#[cfg(feature = "pyo3")]
#[pyfunction]
fn log_info(msg: String) {
    // println!("{msg}");
    info!("{msg}")
}

#[cfg(feature = "pyo3")]
#[pyfunction]
fn log_warn(msg: String) {
    warn!("{msg}")
}

#[cfg(feature = "pyo3")]
#[pyfunction]
fn log_error(msg: String) {
    error!("{msg}")
}

/// A Python module implemented in Rust.
#[cfg(feature = "pyo3")]
#[pymodule]
fn stepper_synth_backend(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use pygame_coms::{
        ADSRState, LfoState, LowPassState, OscState, SynthEngineState, WTSynthParam,
    };
    use sequencer::{PerChannelMidi, SequenceChannel};

    m.add_function(wrap_pyfunction!(log_trace, m)?)?;
    m.add_function(wrap_pyfunction!(log_debug, m)?)?;
    m.add_function(wrap_pyfunction!(log_info, m)?)?;
    m.add_function(wrap_pyfunction!(log_warn, m)?)?;
    m.add_function(wrap_pyfunction!(log_error, m)?)?;

    // m.add_class::<SynthParam>()?;
    // m.add_class::<PythonCmd>()?;
    // m.add_class::<TrackerIPC>()?;
    m.add_class::<Knob>()?;
    m.add_class::<GuiParam>()?;
    m.add_class::<SynthEngineType>()?;
    // m.add_class::<State>()?;
    m.add_class::<OscType>()?;
    m.add_class::<ReverbParam>()?;
    m.add_class::<EffectType>()?;
    m.add_class::<Screen>()?;
    m.add_class::<StepperSynth>()?;
    m.add_class::<StepperSynthState>()?;
    m.add_class::<Param>()?;
    m.add_class::<Step>()?;
    m.add_class::<StepCmd>()?;
    m.add_class::<Sequence>()?;
    m.add_class::<SynthEngineState>()?;
    m.add_class::<OscState>()?;
    m.add_class::<LowPassState>()?;
    m.add_class::<ADSRState>()?;
    m.add_class::<LfoState>()?;
    m.add_class::<WTSynthParam>()?;
    m.add_class::<PerChannelMidi>()?;
    m.add_class::<SequenceChannel>()?;
    // m.add_class::<>()?;

    Ok(())
}
