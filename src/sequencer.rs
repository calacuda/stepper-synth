use fxhash::FxHashSet;
use log::*;
use midi_control::{ControlEvent, KeyEvent, MidiMessage, MidiNote};
use pyo3::prelude::*;
use std::{
    ops::{Index, IndexMut},
    u16,
};

use crate::{synth_engines::Synth, MidiControlled};

pub type MidiMessages = FxHashSet<(u8, StepCmd)>;
pub type MidiControlCode = u8;
pub type MidiInt = u8;

#[pyclass(module = "stepper_synth_backend", get_all)]
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
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

#[pyclass(module = "stepper_synth_backend", get_all)]
#[derive(Debug, Clone, Default)]
pub struct Step {
    pub on_enter: MidiMessages,
    pub on_exit: MidiMessages,
}

#[pyclass(module = "stepper_synth_backend", get_all)]
#[derive(Debug, Clone)]
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

#[pyclass(module = "stepper_synth_backend", get_all)]
#[derive(Debug, Clone, Default)]
pub struct StepperState {
    pub recording: bool,
    pub playing: bool,
}

#[derive(Debug, Clone)]
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

impl SequenceIndex {
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

#[derive(Debug, Clone)]
pub struct SequencerIntake {
    sequences: Vec<Sequence>,
    pub synth: Synth,
    // sequence_i: usize,
    pub rec_head: SequenceIndex,
    pub play_head: SequenceIndex,
    pub state: StepperState,
    pub bpm: u16,
}

impl SequencerIntake {
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
}

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
                    self.state.playing = false;
                    self.state.recording = false;
                }
                118 => {
                    self.state.playing = true;
                    self.state.recording = false;
                }
                119 => {
                    self.state.playing = false;
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
            // MidiMessage::ControlChange(_channel, ControlEvent { control, value: _ }) => {
            //     match control {
            //         115 => {
            //             self.rec_head.step = ((self.rec_head.step as i32 - 1)
            //                 % (self.sequences[self.rec_head.sequence].steps.len() as i32))
            //                 as usize;
            //         }
            //         116 => {
            //             self.rec_head.step += 1;
            //             self.rec_head.step %= self.sequences[self.rec_head.sequence].steps.len();
            //             info!(
            //                 "n steps {}",
            //                 self.sequences[self.rec_head.sequence].steps.len()
            //             );
            //         }
            //         117 => {
            //             self.state.playing = false;
            //             self.state.recording = false;
            //             // return;
            //         }
            //         118 => {
            //             self.state.playing = true;
            //             self.state.recording = false;
            //             // return;
            //         }
            //         119 => {
            //             self.state.playing = false;
            //             self.state.recording = true;
            //             // return;
            //         }
            //         _ => {}
            //     }
            //
            //     return;
            // }
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

// pub fn ch_to_u8(ch: Channel) -> u8 {}
