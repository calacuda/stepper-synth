use anyhow::Result;
use crossbeam::channel::Sender;
use fern::colors::{Color, ColoredLevelConfig};
use ipc::RustIPC;
use ipc::{gen_ipc, TrackerIPC};
use log::*;
use midi_control::ControlEvent;
use midi_control::KeyEvent;
use midi_control::MidiMessage;
use midir::MidiInput;
use midir::{Ignore, PortInfoError};
use pygame_coms::{PythonCmd, State, SynthParam};
use pyo3::prelude::*;
use rodio::OutputStream;
use rodio::Source;
use std::collections::HashMap;
use std::time::Duration;
use std::{
    process::exit,
    sync::{Arc, Mutex},
    thread::spawn,
};
use synth_engines::Synth;
// use synth_rt::synth::Synth;

pub const SAMPLE_RATE: u32 = 44_100;

pub mod effects;
mod ipc;
// mod organ;
mod pygame_coms;
pub mod synth_engines;
// pub mod synth_common;

pub trait SampleGen {
    fn get_sample(&mut self) -> f32;
}

pub trait KnobCtrl {
    fn knob_1(&mut self, value: f32) -> Option<SynthParam>;
    fn knob_2(&mut self, value: f32) -> Option<SynthParam>;
    fn knob_3(&mut self, value: f32) -> Option<SynthParam>;
    fn knob_4(&mut self, value: f32) -> Option<SynthParam>;
    fn knob_5(&mut self, value: f32) -> Option<SynthParam>;
    fn knob_6(&mut self, value: f32) -> Option<SynthParam>;
    fn knob_7(&mut self, value: f32) -> Option<SynthParam>;
    fn knob_8(&mut self, value: f32) -> Option<SynthParam>;
}

pub struct Player {
    pub synth: Arc<Mutex<Synth>>,
}

impl Iterator for Player {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // println!("yet to lock");
        let sample = self.synth.lock().expect("couldn't lock synth").get_sample();
        // println!("locked");
        Some(sample)
    }
}

impl Source for Player {
    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn send_mesg(tx: &Sender<State>, msg: State) {
    if let Err(e) = tx.send(msg) {
        error!("failed to send \"{msg:?}\" to python frontend because: {e}.");
    }
}

fn run_midi(synth: Arc<Mutex<Synth>>, my_ipc: RustIPC) -> Result<()> {
    let tx = my_ipc.tx;
    let mut registered_ports = HashMap::new();

    loop {
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
            let tx = tx.clone();

            // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
            registered_ports.insert(
                port_name,
                midi_in.connect(
                    in_port,
                    "midir-read-input",
                    move |_stamp, message, _| {
                        // println!("{}: {:?} (len = {})", stamp, message, message.len());
                        let message = MidiMessage::from(message);
                        // do midi stuff

                        // println!("midi messagze => {message:?} on port {port_name:?}");
                        let send = |msg: State| send_mesg(&tx, msg);

                        match message {
                            MidiMessage::Invalid => {
                                // info!("midi_cmd_buf => {message:?}");
                                error!("midi_cmd -> {message:?}");
                                // info!("midi cmd => {:?}", MidiMessage::from(message));
                                error!("midi command invalid");
                            }
                            MidiMessage::NoteOn(_, KeyEvent { key, value }) => {
                                println!("playing note: {key}");
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
                                    send(SynthParam::PitchBend(bend));
                                } else {
                                    synth.lock().unwrap().engine.unbend();
                                    send(SynthParam::PitchBend(0.0));
                                }
                            }
                            MidiMessage::ControlChange(_, ControlEvent { control, value }) => {
                                let value = value as f32 / 127.0;

                                match control {
                                    70 => {
                                        // synth.lock().unwrap().set_atk(value);
                                        // send(SynthParam::Atk(value));
                                        synth.lock().unwrap().engine.knob_1(value)
                                    }
                                    71 => {
                                        // synth.lock().unwrap().set_decay(value);
                                        // send(SynthParam::Dcy(value));
                                        synth.lock().unwrap().engine.knob_2(value)
                                    }
                                    72 => {
                                        // synth.lock().unwrap().set_sus(value);
                                        // send(SynthParam::Sus(value));
                                        synth.lock().unwrap().engine.knob_3(value)
                                    }
                                    73 => {
                                        // synth.lock().unwrap().set_release(value);
                                        // send(SynthParam::Rel(value));
                                        synth.lock().unwrap().engine.knob_4(value)
                                    }

                                    74 => {
                                        // synth.lock().unwrap().set_cutoff(value);
                                        // send(SynthParam::FilterCutoff(value));
                                        synth.lock().unwrap().engine.knob_5(value)
                                    }
                                    75 => {
                                        // synth.lock().unwrap().set_re.sonace(value);
                                        // send(SynthParam::FilterRes(value));
                                        synth.lock().unwrap().engine.knob_6(value)
                                    }
                                    76 => {
                                        // synth.lock().unwrap().set_chorus_depth(value);
                                        // send(SynthParam::DelayVol(value));
                                        synth.lock().unwrap().engine.knob_7(value)
                                    }
                                    77 => {
                                        // synth.lock().unwrap().set_chorus_speed(value);
                                        // send(SynthParam::DelayTime(value));
                                        synth.lock().unwrap().engine.knob_8(value)
                                    }
                                    1 => {
                                        // synth.lock().unwrap().set_leslie_speed(value);
                                        // send(SynthParam::SpeakerSpinSpeed(value));
                                        synth.lock().unwrap().engine.volume_swell(value);
                                        None
                                    }
                                    _ => None,
                                }
                                .map(send);
                            }
                            _ => {}
                        }
                    },
                    (),
                ),
            );
        }

        let rx = my_ipc.rx.clone();

        // loop {
        if let Ok(command) = rx.try_recv() {
            match command {
                PythonCmd::SetSynthParam(param) => match param {
                    // TODO: rework the commands that the front end will send

                    // SynthParam::Atk(value) => synth.lock().unwrap().set_atk(value),
                    // SynthParam::Dcy(value) => synth.lock().unwrap().set_decay(value),
                    // SynthParam::Sus(value) => synth.lock().unwrap().set_sus(value),
                    // SynthParam::Rel(value) => synth.lock().unwrap().set_release(value),
                    // SynthParam::FilterCutoff(value) => synth.lock().unwrap().set_cutoff(value),
                    // SynthParam::FilterRes(value) => synth.lock().unwrap().set_resonace(value),
                    // SynthParam::DelayVol(value) => synth.lock().unwrap().set_chorus_depth(value),
                    // SynthParam::DelayTime(value) => synth.lock().unwrap().set_chorus_speed(value),
                    // SynthParam::SpeakerSpinSpeed(value) => {
                    //     synth.lock().unwrap().set_leslie_speed(value)
                    // }
                    // SynthParam::PitchBend(_) => {
                    //     error!("gui can not set pitch bend")
                    // }
                    _ => {}
                },
                PythonCmd::Exit() => {
                    return Ok(());
                }
            }
        }
        // }
    }
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

/// Formats the sum of two numbers as string.
#[pyfunction]
fn start_audio() -> PyResult<TrackerIPC> {
    let (my_ipc, py_ipc) = gen_ipc();

    // build synth in arc mutex
    let synth = Arc::new(Mutex::new(Synth::new()));

    // synth.lock().unwrap().engine.set_volume(1.0);

    let output = Player {
        synth: synth.clone(),
    };

    spawn(move || {
        let res = logger_init();

        if let Err(reason) = res {
            eprintln!("failed to initiate logger because {reason}");
        } else {
            log::debug!("logger initiated");
        }

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();

        // start output
        if let Err(e) = stream_handle.play_raw(output) {
            error!("[ERROR] => {e}");
            exit(1);
        } else {
            info!("midi initialized");
        }

        if let Err(e) = run_midi(synth, my_ipc) {
            error!("{e}");
        }
    });

    println!("run_midi called");

    Ok(py_ipc)
}

/// A Python module implemented in Rust.
#[pymodule]
fn stepper_synth_backend(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_audio, m)?)?;

    m.add_class::<SynthParam>()?;
    m.add_class::<PythonCmd>()?;
    m.add_class::<TrackerIPC>()?;
    Ok(())
}
