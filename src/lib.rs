use anyhow::Result;
use effects::reverb::ReverbParam;
use effects::EffectType;
use fern::colors::{Color, ColoredLevelConfig};
use fxhash::FxHashMap;
// use ipc::TrackerIPC;
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
// pub mod ipc;
pub mod pygame_coms;
pub mod synth_engines;

pub trait SampleGen {
    fn get_sample(&mut self) -> f32;
}

// TODO: make python ogject use try_lock() in a loop so it doesn't disturb audio gen.

#[allow(unused_variables)]
pub trait KnobCtrl {
    // TODO: have these return a bool, representing if the system should send a state update
    // message to the python front end.

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

// fn send_mesg(tx: &Sender<State>, msg: State) {
//     if let Err(e) = tx.send(msg.clone()) {
//         error!("failed to send \"{msg:?}\" to python frontend because: {e}.");
//     }
// }

fn run_midi(
    synth: Arc<Mutex<Synth>>,
    updated: Arc<Mutex<bool>>,
    exit: Arc<AtomicBool>,
) -> Result<()> {
    // let tx = my_ipc.tx;
    let mut registered_ports = HashMap::default();

    // send_mesg(&tx, synth.lock().unwrap().get_state());

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
        // .filter(|metadata| metadata.target().starts_with("stepper"))
        .chain(std::io::stderr())
        .chain(fern::log_file("output.log")?)
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
        // .filter(|metadata| metadata.target().starts_with("stepper"))
        .chain(std::io::stderr())
        .chain(fern::log_file("output.log")?)
        .apply()?;

    Ok(())
}

// /// Formats the sum of two numbers as string.
// #[pyfunction]
// fn start_audio() -> PyResult<(TrackerIPC, State)> {
//     let (my_ipc, py_ipc) = gen_ipc();
//
//     // build synth in arc mutex
//     let synth = Arc::new(Mutex::new(Synth::new()));
//
//     {
//         let s = synth.clone();
//
//         spawn(move || {
//             let res = logger_init();
//
//             if let Err(reason) = res {
//                 eprintln!("failed to initiate logger because {reason}");
//             } else {
//                 log::debug!("logger initiated");
//             }
//
//             let params = OutputDeviceParameters {
//                 channels_count: 1,
//                 sample_rate: SAMPLE_RATE as usize,
//                 // channel_sample_count: 2048,
//                 channel_sample_count: 1024,
//             };
//             let device = {
//                 let s = s.clone();
//
//                 run_output_device(params, move |data| {
//                     for samples in data.chunks_mut(params.channels_count) {
//                         let value = s.lock().expect("couldn't lock synth").get_sample();
//
//                         for sample in samples {
//                             *sample = value;
//                         }
//                     }
//                 })
//             };
//
//             if let Err(e) = device {
//                 error!("strating audio playback caused error: {e}");
//             }
//
//             if let Err(e) = run_midi(s, my_ipc) {
//                 error!("{e}");
//             }
//         });
//     }
//
//     println!("run_midi called");
//
//     Ok((py_ipc, synth.clone().lock().unwrap().get_state()))
// }

/// A Python module implemented in Rust.
#[pymodule]
fn stepper_synth_backend(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(start_audio, m)?)?;

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
