use midi_control::{ControlEvent, KeyEvent, MidiMessage, MidiNote};
use pyo3::prelude::*;
use std::ops::{Index, IndexMut};

use crate::MidiControlled;

pub type MidiMessages = Vec<(u8, StepCmd)>;
pub type MidiControlCode = u8;
pub type MidiInt = u8;

#[pyclass(module = "stepper_synth_backend", get_all)]
#[derive(Debug, Clone)]
pub enum StepCmd {
    Play {
        note: MidiNote,
        vel: u8,
    },
    Stop {
        note: MidiNote,
        vel: u8,
    },
    CC {
        code: MidiControlCode,
        value: MidiInt,
    },
}

#[pyclass(module = "stepper_synth_backend", get_all)]
#[derive(Debug, Clone)]
pub struct Step {
    on_enter: MidiMessages,
    on_exit: MidiMessages,
}

#[pyclass(module = "stepper_synth_backend", get_all)]
#[derive(Debug, Clone)]
pub struct Sequence {
    pub human_name: Option<String>,
    steps: Vec<Step>,
}

impl Default for Sequence {
    fn default() -> Self {
        Self {
            human_name: None,
            steps: Vec::with_capacity(16),
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
    fn next_sequence(&mut self) {
        self.sequence += 1;
        self.step = 0;
        // self.on_enter = true;
    }

    fn prev_sequence(&mut self) {
        self.sequence -= 1;
        self.step = 0;
        // self.on_enter = true;
    }
}

#[derive(Debug, Clone)]
pub struct SequencerIntake {
    sequences: Vec<Sequence>,
    // sequence_i: usize,
    pub rec_head: SequenceIndex,
    pub play_head: SequenceIndex,
    pub state: StepperState,
}

impl SequencerIntake {
    pub fn new() -> Self {
        Self {
            sequences: vec![Sequence::default()],
            // sequence_i: 0,
            rec_head: SequenceIndex::default(),
            play_head: SequenceIndex::default(),
            state: StepperState::default(),
        }
    }
    //
    // pub fn next_sequence(&mut self) {
    //     let i = self.sequence_i;
    //     let len = self.sequences.len();
    //
    //     self.sequence_i = ((i as i64 + 1) % len as i64) as usize;
    // }
    //
    // pub fn prev_sequence(&mut self) {
    //     let i = self.sequence_i;
    //     let len = self.sequences.len();
    //
    //     self.sequence_i = ((i as i64 - 1) % len as i64) as usize;
    // }

    pub fn new_sequence(&mut self) {
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

    pub fn get_sequence(&self, i: usize) -> Sequence {
        self.sequences[i].clone()
    }
}

impl MidiControlled for SequencerIntake {
    fn midi_input(&mut self, message: &MidiMessage) {
        let (ch, msg, on_enter) = match *message {
            MidiMessage::NoteOn(channel, KeyEvent { key, value }) => {
                let ch = channel as u8;

                (
                    ch,
                    StepCmd::Play {
                        note: key,
                        vel: value,
                    },
                    true,
                )
            }
            MidiMessage::NoteOff(channel, KeyEvent { key, value }) => {
                let ch = channel as u8;

                (
                    ch,
                    StepCmd::Stop {
                        note: key,
                        vel: value,
                    },
                    false,
                )
            }
            MidiMessage::PitchBend(cahnnel, lsb, msb) => return,
            MidiMessage::ControlChange(_channel, ControlEvent { control, value: _ }) => {
                // let ch = channel as u8;

                match control {
                    115 => {
                        self.rec_head.step += 1;
                        self.rec_head.step %= self.sequences[self.rec_head.sequence].steps.len();
                        return;
                    }
                    116 => {
                        self.rec_head.step = ((self.rec_head.step as i32 - 1)
                            % (self.sequences[self.rec_head.sequence].steps.len() as i32))
                            as usize;
                        return;
                    }
                    117 => {
                        self.state.playing = false;
                        self.state.recording = false;
                        return;
                    }
                    118 => {
                        self.state.playing = true;
                        self.state.recording = false;
                        return;
                    }
                    119 => {
                        self.state.playing = false;
                        self.state.recording = true;
                        return;
                    }
                    _ => return, //         _ => (
                                 //             ch,
                                 //             StepCmd::CC {
                                 //                 code: control,
                                 //                 value,
                                 //             },
                                 //             true,
                                 //         ),
                }
            }
            _ => {
                return;
            }
        };

        if on_enter {
            self.sequences[self.rec_head.clone()]
                .on_enter
                .push((ch, msg));
        } else {
            self.sequences[self.rec_head.clone()]
                .on_exit
                .push((ch, msg));
        }
    }
}

// pub fn ch_to_u8(ch: Channel) -> u8 {}
