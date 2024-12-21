use anyhow::Result;
use effects::reverb::ReverbParam;
use effects::EffectType;
use fern::colors::{Color, ColoredLevelConfig};
use fxhash::FxHashMap;
use log::*;
use midi_control::ControlEvent;
use midi_control::KeyEvent;
use midi_control::MidiMessage;
use midir::MidiInput;
use midir::{Ignore, PortInfoError};
use pygame_coms::Screen;
use pygame_coms::StepperSynth;
use pygame_coms::StepperSynthState;
use pygame_coms::{GuiParam, Knob, SynthEngineType, SynthParam};
use pyo3::prelude::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use synth_engines::synth::OscType;
use synth_engines::Synth;

pub type HashMap<Key, Val> = FxHashMap<Key, Val>;

pub const SAMPLE_RATE: u32 = 48_000;

pub mod effects;
pub mod pygame_coms;
pub mod synth_engines;

pub trait SampleGen {
    fn get_sample(&mut self) -> f32;
}

#[allow(unused_variables)]
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

fn run_midi(
    synth: Arc<Mutex<Synth>>,
    updated: Arc<Mutex<bool>>,
    exit: Arc<AtomicBool>,
) -> Result<()> {
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
            let synth = synth.clone();
            // let tx = tx.clone();
            let updated = updated.clone();

            registered_ports.insert(
                port_name,
                midi_in.connect(
                    in_port,
                    "midir-read-input",
                    move |_stamp, message, _| {
                        let message = MidiMessage::from(message);
                        let send = || {
                            let mut u = updated.lock().unwrap();
                            *u = true;
                        };

                        // do midi stuff
                        match message {
                            MidiMessage::Invalid => {
                                error!("system recieved an invalid MIDI message.");
                            }
                            MidiMessage::NoteOn(_, KeyEvent { key, value }) => {
                                debug!("playing note: {key}");
                                synth.lock().unwrap().engine.play(key, value)
                            }
                            MidiMessage::NoteOff(_, KeyEvent { key, value: _ }) => {
                                synth.lock().unwrap().engine.stop(key)
                            }
                            MidiMessage::PitchBend(_, lsb, msb) => {
                                let bend =
                                    i16::from_le_bytes([lsb, msb]) as f32 / (32_000.0 * 0.5) - 1.0;

                                if bend > 0.026 || bend < -0.026 {
                                    synth.lock().unwrap().engine.bend(bend);
                                    send();
                                } else {
                                    synth.lock().unwrap().engine.unbend();
                                    // send(SynthParam::PitchBend(0.0));
                                    send();
                                }
                            }
                            MidiMessage::ControlChange(_, ControlEvent { control, value }) => {
                                let value = value as f32 / 127.0;

                                if match control {
                                    70 => synth.lock().unwrap().engine.knob_1(value),
                                    71 => synth.lock().unwrap().engine.knob_2(value),
                                    72 => synth.lock().unwrap().engine.knob_3(value),
                                    73 => synth.lock().unwrap().engine.knob_4(value),
                                    74 => synth.lock().unwrap().engine.knob_5(value),
                                    75 => synth.lock().unwrap().engine.knob_6(value),
                                    76 => synth.lock().unwrap().engine.knob_7(value),
                                    77 => synth.lock().unwrap().engine.knob_8(value),
                                    1 => synth.lock().unwrap().engine.volume_swell(value),
                                    _ => false,
                                } {
                                    send()
                                }
                            }
                            _ => {}
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
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .chain(fern::log_file("stepper-synth.log")?)
        // .filter(|metadata| metadata..starts_with("stepper"))
        .chain(std::io::stderr())
        .apply()?;

    #[cfg(not(debug_assertions))]
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] {}",
                colors.color(record.level()),
                message
            ))
        })
        .chain(fern::log_file("stepper-synth.log")?)
        // .filter(|metadata| metadata.target().starts_with("stepper"))
        .chain(std::io::stderr())
        .apply()?;

    info!("logger started");

    Ok(())
}

#[pyfunction]
fn log_trace(msg: String) {
    // println!("{msg}");
    trace!("{msg}")
}

#[pyfunction]
fn log_debug(msg: String) {
    debug!("{msg}")
}

#[pyfunction]
fn log_info(msg: String) {
    // println!("{msg}");
    info!("{msg}")
}

#[pyfunction]
fn log_warn(msg: String) {
    warn!("{msg}")
}

#[pyfunction]
fn log_error(msg: String) {
    error!("{msg}")
}

/// A Python module implemented in Rust.
#[pymodule]
fn stepper_synth_backend(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(start_audio, m)?)?;
    m.add_function(wrap_pyfunction!(log_trace, m)?)?;
    m.add_function(wrap_pyfunction!(log_debug, m)?)?;
    m.add_function(wrap_pyfunction!(log_info, m)?)?;
    m.add_function(wrap_pyfunction!(log_warn, m)?)?;
    m.add_function(wrap_pyfunction!(log_error, m)?)?;

    m.add_class::<SynthParam>()?;
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
    // m.add_class::<>()?;

    Ok(())
}
