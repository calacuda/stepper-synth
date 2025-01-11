use crate::{
    pygame_coms::SynthEngineType,
    synth_engines::{Synth, SynthEngine},
    HashSet, MidiControlled,
};
use log::*;
use midi_control::{ControlEvent, KeyEvent, MidiMessage, MidiNote};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Index, IndexMut},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
    u16,
};
use strum::IntoEnumIterator;

pub type MidiMessages = HashSet<(u8, StepCmd)>;
pub type MidiControlCode = u8;
pub type MidiInt = u8;

#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize)]
pub enum StepCmd {
    Play {
        note: MidiNote,
        vel: u8,
    },
    Stop {
        note: MidiNote,
        // vel: u8,
    },
    CC {
        code: MidiControlCode,
        value: MidiInt,
    },
}

#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Step {
    pub on_enter: MidiMessages,
    pub on_exit: MidiMessages,
}

#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sequence {
    pub human_name: Option<String>,
    pub steps: Vec<Step>,
}

impl Default for Sequence {
    fn default() -> Self {
        let steps: Vec<Step> = (0..16).map(|_| Step::default()).collect();
        // steps[0]
        //     .on_enter
        //     .insert((0, StepCmd::Play { note: 60, vel: 50 }));

        Self {
            human_name: None,
            steps,
        }
    }
}

// #[pyclass(module = "stepper_synth_backend", get_all)]
#[derive(Debug, Default)]
pub struct StepperState {
    pub recording: bool,
    pub playing: AtomicBool,
}

#[derive(Debug, Clone, Copy)]
pub struct SequenceIndex {
    sequence: usize,
    step: usize,
    // on_enter: bool,
}

impl Default for SequenceIndex {
    fn default() -> Self {
        Self {
            sequence: 0,
            step: 0,
            // on_enter: true,
        }
    }
}

impl SequenceIndex {
    pub fn get_sequence(&self) -> usize {
        self.sequence
    }

    fn set_sequence(&mut self, sequence: usize) {
        self.sequence = sequence
    }

    pub fn next_sequence(&mut self) {
        self.sequence += 1;
        self.step = 0;
        // self.on_enter = true;
    }

    pub fn prev_sequence(&mut self) {
        self.sequence -= 1;
        self.step = 0;
        // self.on_enter = true;
    }
}

impl Index<SequenceIndex> for Vec<Sequence> {
    type Output = Step;

    fn index(&self, index: SequenceIndex) -> &Self::Output {
        // if index.on_enter {
        //     &self[index.sequence].steps[index.step].on_enter
        // } else {
        //     &self[index.sequence].steps[index.step].on_exit
        // }
        &self[index.sequence].steps[index.step]
    }
}

impl IndexMut<SequenceIndex> for Vec<Sequence> {
    fn index_mut(&mut self, index: SequenceIndex) -> &mut Self::Output {
        // if index.on_enter {
        //     &mut self[index.sequence].steps[index.step].on_enter
        // } else {
        //     &mut self[index.sequence].steps[index.step].on_exit
        // }
        &mut self[index.sequence].steps[index.step]
    }
}

#[derive(Debug)]
pub struct SequencerIntake {
    sequences: Vec<Sequence>,
    #[cfg(feature = "pyo3")]
    pub synth: Synth,
    // sequence_i: usize,
    pub rec_head: SequenceIndex,
    pub play_head: SequenceIndex,
    pub state: StepperState,
    pub bpm: u16,
}

impl SequencerIntake {
    #[cfg(feature = "pyo3")]
    pub fn new(synth: Synth) -> Self {
        Self {
            sequences: vec![
                Sequence::default(),
                Sequence::default(),
                Sequence::default(),
                Sequence::default(),
            ],
            // sequence_i: 0,
            rec_head: SequenceIndex::default(),
            play_head: SequenceIndex::default(),
            state: StepperState::default(),
            bpm: 120,
            synth,
        }
    }

    #[cfg(not(feature = "pyo3"))]
    pub fn new() -> Self {
        Self {
            sequences: vec![
                Sequence::default(),
                Sequence::default(),
                Sequence::default(),
                Sequence::default(),
            ],
            // sequence_i: 0,
            rec_head: SequenceIndex::default(),
            play_head: SequenceIndex::default(),
            state: StepperState::default(),
            bpm: 120,
        }
    }

    pub fn get_step(&self, play: bool) -> Step {
        let i = if play {
            self.play_head.clone()
        } else {
            self.rec_head.clone()
        };

        self.sequences[i].clone()
    }

    pub fn get_cursor(&self, play: bool) -> usize {
        if play {
            self.play_head.step
        } else {
            self.rec_head.step
        }
    }

    pub fn add_step(&mut self) {
        self.sequences[self.rec_head.sequence]
            .steps
            .push(Step::default());
    }

    pub fn del_step(&mut self) {
        self.sequences[self.rec_head.sequence].steps.pop();
    }

    pub fn next_sequence(&mut self) {
        let len = self.sequences.len();

        // info!("rec head sequence = {}", self.play_head.sequence);
        self.rec_head.sequence = ((self.rec_head.sequence as i64 + 1) % (len as i64)) as usize;
        // info!(
        //     "rec head sequence = {}, len = {}",
        //     self.play_head.sequence, len
        // );
        // }
    }

    pub fn prev_sequence(&mut self) {
        let len = self.sequences.len();

        if self.rec_head.sequence == 0 {
            self.rec_head.sequence = len - 1;
        } else {
            self.rec_head.sequence -= 1;
        }
        // info!("rec head sequence = {}, {len}", self.play_head.sequence);
    }

    pub fn next_step(&mut self) {
        let len = self.sequences[self.rec_head.sequence].steps.len();

        // info!("rec head sequence = {}", self.play_head.sequence);
        self.rec_head.step = ((self.rec_head.step as i64 + 1) % (len as i64)) as usize;
        // info!(
        //     "rec head sequence = {}, len = {}",
        //     self.play_head.sequence, len
        // );
        // }
    }

    pub fn prev_step(&mut self) {
        // let len = self.sequences.len();
        let len = self.sequences[self.rec_head.sequence].steps.len();

        if self.rec_head.step == 0 {
            self.rec_head.step = len - 1;
        } else {
            self.rec_head.step -= 1;
        }
        // info!("rec head sequence = {}, {len}", self.play_head.sequence);
    }

    pub fn get_name(&self) -> String {
        if let Some(name) = self.sequences[self.rec_head.sequence].human_name.clone() {
            name
        } else {
            format!("{}", self.rec_head.sequence)
        }
    }

    pub fn new_sequence(&mut self) {
        info!("adding new sequence");
        self.sequences.push(Sequence::default());
    }

    pub fn del_sequence(&mut self, at: usize) {
        if at >= self.sequences.len() {
            return;
        }

        if at <= self.rec_head.sequence {
            self.rec_head.sequence -= 1;
        }

        if at <= self.play_head.sequence {
            self.play_head.sequence -= 1;
        }

        self.sequences = self
            .sequences
            .iter()
            .enumerate()
            .filter_map(|(i, seq)| if i != at { Some(seq.clone()) } else { None })
            .collect();
    }

    pub fn get_sequence(&self) -> Sequence {
        // self.sequences[i].clone()
        self.sequences[self.rec_head.sequence].clone()
    }

    pub fn set_rec_head_seq(&mut self, seq: i64) {
        self.rec_head.sequence = (seq % self.sequences.len() as i64) as usize;
    }

    pub fn set_sequence(&mut self, sequence: usize) {
        if sequence < self.sequences.len() {
            self.rec_head.set_sequence(sequence);
        } else {
            error!("atempted to set record head to {sequence}, but that sequence doesn't exist.");
        }
    }
}

#[cfg(feature = "pyo3")]
impl MidiControlled for SequencerIntake {
    fn midi_input(&mut self, message: &MidiMessage) {
        self.synth.midi_input(message);

        if let MidiMessage::ControlChange(_channel, ControlEvent { control, value: _ }) = message {
            match control {
                115 => {
                    self.rec_head.step = if self.rec_head.step > 0 {
                        self.rec_head.step - 1
                    } else {
                        self.sequences[self.rec_head.sequence].steps.len() - 1
                    };
                }
                116 => {
                    self.rec_head.step += 1;
                    self.rec_head.step %= self.sequences[self.rec_head.sequence].steps.len();
                }
                117 => {
                    self.state.playing.store(false, Ordering::Relaxed);
                    self.state.recording = false;
                }
                118 => {
                    self.state.playing.store(true, Ordering::Relaxed);
                    self.state.recording = false;
                    info!("setting playing to true");
                }
                119 => {
                    self.state.playing.store(false, Ordering::Relaxed);
                    self.state.recording = true;
                }
                _ => {}
            }
        }

        if !self.state.recording {
            return;
        }

        let (ch, msg, on_enter) = match *message {
            MidiMessage::NoteOn(channel, KeyEvent { key, value }) => {
                let ch = channel as u8;
                let cmd = StepCmd::Play {
                    note: key,
                    vel: value,
                };

                // if self.sequences[self.rec_head.clone()].on_enter.iter().filter_map(| | ) {}

                (ch, cmd, true)
            }
            MidiMessage::NoteOff(channel, KeyEvent { key, value: _ }) => {
                let ch = channel as u8;

                (
                    ch,
                    StepCmd::Stop {
                        note: key,
                        // vel: value,
                    },
                    false,
                )
            }
            MidiMessage::PitchBend(_cahnnel, _lsb, _msb) => return,
            _ => {
                return;
            }
        };

        let step = if on_enter {
            &mut self.sequences[self.rec_head.clone()].on_enter
        } else {
            &mut self.sequences[self.rec_head.clone()].on_exit
        };

        let step_filter_f =
            |(rec_ch, rec_msg)| {
                if ch == rec_ch {
                    match (rec_msg, msg.clone()) {
                        (
                            StepCmd::Play { note: n1, vel: _ },
                            StepCmd::Play { note: n2, vel: _ },
                        ) if n1 == n2 => Some(()),
                        (
                            StepCmd::CC { code: c1, value: _ },
                            StepCmd::CC { code: c2, value: _ },
                        ) if c1 == c2 => Some(()),
                        (StepCmd::Stop { note: n1 }, StepCmd::Stop { note: n2 }) if n1 == n2 => {
                            Some(())
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            };

        let step_not_contains = step
            .clone()
            .into_iter()
            .filter_map(step_filter_f)
            .collect::<Vec<()>>()
            .len()
            == 0;

        if step_not_contains {
            step.insert((ch, msg));
        } else {
            // info!("rm'ing a message");
            // info!("rm'ing a message from on_enter {on_enter}");
            // info!("number of messages before = {}", step.len());
            step.retain(|msg| step_filter_f(msg.clone()).is_none());
            // info!("number of messages after = {}", step.len());
        }
    }
}

#[cfg(feature = "pyo3")]
pub fn play_sequence(seq: Arc<Mutex<SequencerIntake>>) {
    let mut beat_time = Duration::from_secs_f64(60.0 / seq.lock().unwrap().bpm as f64);
    // let mut last_on_exit = HashSet::default();
    let synth_types: Vec<SynthEngineType> = SynthEngineType::iter().collect();
    let mut playing: HashSet<(u8, u8)> = HashSet::default();

    let mut send_midi = |synth: &mut Synth, midi_s: MidiMessages| {
        for midi in midi_s {
            let instrument = if midi.0 == 0 {
                synth.get_engine()
            } else if let Some(synth_type) = synth_types.get((midi.0 - 1) as usize) {
                synth.engines.index_mut(*synth_type as usize)
            } else {
                continue;
            };

            match midi.1 {
                StepCmd::Play { note, vel } => {
                    playing.insert((midi.0, note));

                    instrument.play(note, vel)
                }
                StepCmd::Stop { note } => {
                    playing.remove(&(midi.0, note));

                    instrument.stop(note)
                }
                StepCmd::CC { code: _, value: _ } => {}
            }
        }
    };

    let mut play_step = |last_on_exit: MidiMessages| {
        // info!("beat");
        let mut seq = seq.lock().unwrap();
        // info!("after sequence lock");
        let step = seq.sequences[seq.play_head].clone();
        send_midi(&mut seq.synth, last_on_exit);

        send_midi(&mut seq.synth, step.on_enter);
        step.on_exit.clone()
    };
    let inc_step = || {
        // info!("beat");
        let mut seq = seq.lock().unwrap();
        seq.play_head.step += 1;
        seq.play_head.step %= seq.sequences[seq.play_head.sequence].steps.len();
    };

    let mut last_on_exit = play_step(HashSet::default());
    let mut last_play = Instant::now();

    while seq
        .clone()
        .lock()
        .unwrap()
        .state
        .playing
        .load(Ordering::Relaxed)
    {
        if last_play.elapsed() >= beat_time {
            inc_step();
            last_on_exit = play_step(last_on_exit);

            beat_time = Duration::from_secs_f64(60.0 / seq.lock().unwrap().bpm as f64);
            last_play = Instant::now();
        }
    }

    let mut seq = seq.lock().unwrap();
    seq.play_head.step = 0;
    playing.into_iter().for_each(|(ch, note)| {
        let synth = &mut seq.synth;

        if ch == 0 {
            synth.get_engine().stop(note);
        } else if let Some(synth_type) = synth_types.get((ch - 1) as usize) {
            synth.engines.index_mut(*synth_type as usize).stop(note);
        }
    })
}
