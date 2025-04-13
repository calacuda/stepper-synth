use crate::{
    pygame_coms::SynthEngineType,
    synth_engines::{Synth, SynthEngine},
    HashSet, MidiControlled,
};
use log::*;
use midi_control::{Channel, ControlEvent, KeyEvent, MidiMessage, MidiNote};
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
    u16, usize,
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
pub struct PerChannelMidi {
    pub channel_a: MidiMessages,
    pub channel_b: MidiMessages,
    pub channel_c: MidiMessages,
    pub channel_d: MidiMessages,
}

impl Index<SequenceChannel> for PerChannelMidi {
    type Output = MidiMessages;

    fn index(&self, index: SequenceChannel) -> &Self::Output {
        match index {
            SequenceChannel::A => &self.channel_a,
            SequenceChannel::B => &self.channel_b,
            SequenceChannel::C => &self.channel_c,
            SequenceChannel::D => &self.channel_d,
        }
    }
}

impl IndexMut<SequenceChannel> for PerChannelMidi {
    fn index_mut(&mut self, index: SequenceChannel) -> &mut Self::Output {
        match index {
            SequenceChannel::A => &mut self.channel_a,
            SequenceChannel::B => &mut self.channel_b,
            SequenceChannel::C => &mut self.channel_c,
            SequenceChannel::D => &mut self.channel_d,
        }
    }
}

impl PerChannelMidi {
    pub fn by_channel(&self) -> Vec<MidiMessages> {
        vec![
            self.channel_a.clone(),
            self.channel_b.clone(),
            self.channel_c.clone(),
            self.channel_d.clone(),
        ]
    }
}

#[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Step {
    pub on_enter: PerChannelMidi,
    pub on_exit: PerChannelMidi,
}

// #[cfg_attr(feature = "pyo3", pyclass(module = "stepper_synth_backend", get_all))]
// #[derive(Debug, Clone, Default, Serialize, Deserialize)]
// pub struct Step {
//     pub ca_step: ChannelStep,
//     pub cb_step: ChannelStep,
//     pub cc_step: ChannelStep,
//     pub cd_step: ChannelStep,
// }

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

#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "stepper_synth_backend", get_all, eq, eq_int)
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SequenceChannel {
    A,
    B,
    C,
    D,
}

impl Into<u8> for SequenceChannel {
    fn into(self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::D => 3,
        }
    }
}

impl From<usize> for SequenceChannel {
    fn from(value: usize) -> Self {
        if value == 0 {
            Self::A
        } else if value == 1 {
            Self::B
        } else if value == 2 {
            Self::C
        } else {
            Self::D
        }
    }
}

#[cfg(feature = "pyo3")]
#[cfg_attr(feature = "pyo3", pymethods)]
impl SequenceChannel {
    fn matches(&self, str_name: String) -> bool {
        match (*self, str_name.to_lowercase().as_str()) {
            (Self::A, "a") => true,
            (Self::B, "b") => true,
            (Self::C, "c") => true,
            (Self::D, "d") => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SequenceIndex {
    sequence: usize,
    step: usize,
    channel: SequenceChannel,
}

impl Default for SequenceIndex {
    fn default() -> Self {
        Self {
            sequence: 0,
            step: 0,
            channel: SequenceChannel::A,
        }
    }
}

impl SequenceIndex {
    pub fn get_sequence(&self) -> usize {
        self.sequence
    }

    pub fn get_channel(&self) -> SequenceChannel {
        self.channel
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

    pub fn set_channel(&mut self, channel: SequenceChannel) {
        self.channel = channel;
    }

    pub fn prev_channel(&mut self) {
        self.channel = match self.channel.clone() {
            SequenceChannel::A => SequenceChannel::D,
            SequenceChannel::B => SequenceChannel::A,
            SequenceChannel::C => SequenceChannel::B,
            SequenceChannel::D => SequenceChannel::C,
        }
    }

    pub fn next_channel(&mut self) {
        self.channel = match self.channel.clone() {
            SequenceChannel::A => SequenceChannel::B,
            SequenceChannel::B => SequenceChannel::C,
            SequenceChannel::C => SequenceChannel::D,
            SequenceChannel::D => SequenceChannel::A,
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

#[derive(Debug)]
pub struct SequencerIntake {
    sequences: Vec<Sequence>,
    #[cfg(feature = "pyo3")]
    pub synth: Synth,
    // sequence_i: usize,
    pub rw_head: SequenceIndex,
    pub view_head: SequenceIndex,
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
            rw_head: SequenceIndex::default(),
            view_head: SequenceIndex::default(),
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
            rw_head: SequenceIndex::default(),
            view_head: SequenceIndex::default(),
            state: StepperState::default(),
            bpm: 120,
        }
    }

    pub fn get_step(&self, play: bool) -> Step {
        let i = if play {
            self.view_head.clone()
        } else {
            self.rw_head.clone()
        };

        self.sequences[i].clone()
    }

    pub fn get_cursor(&self, play: bool) -> usize {
        if play {
            self.view_head.step
        } else {
            self.rw_head.step
        }
    }

    pub fn add_step(&mut self) {
        self.sequences[self.rw_head.sequence]
            .steps
            .push(Step::default());
    }

    pub fn del_step(&mut self) {
        self.sequences[self.rw_head.sequence].steps.pop();
    }

    pub fn next_sequence(&mut self) {
        let len = self.sequences.len();

        // info!("rec head sequence = {}", self.play_head.sequence);
        self.rw_head.sequence = ((self.rw_head.sequence as i64 + 1) % (len as i64)) as usize;
        // info!(
        //     "rec head sequence = {}, len = {}",
        //     self.play_head.sequence, len
        // );
        // }
    }

    pub fn prev_sequence(&mut self) {
        let len = self.sequences.len();

        if self.rw_head.sequence == 0 {
            self.rw_head.sequence = len - 1;
        } else {
            self.rw_head.sequence -= 1;
        }

        // if self.view_head.sequence == 0 {
        //     self.view_head.sequence = len - 1;
        // } else {
        //     self.view_head.sequence -= 1;
        // }

        // info!("rec head sequence = {}, {len}", self.play_head.sequence);
    }

    pub fn next_step(&mut self) {
        let len = self.sequences[self.rw_head.sequence].steps.len();

        // info!("rw_head step = {}", self.rw_head.step);
        self.rw_head.step = ((self.rw_head.step as i64 + 1) % (len as i64)) as usize;
        // self.view_head.step = ((self.rw_head.step as i64 + 1) % (len as i64)) as usize;
        // info!("rw head step = {}, len = {}", self.rw_head.step, len);
        // }
    }

    pub fn prev_step(&mut self) {
        // let len = self.sequences.len();
        let len = self.sequences[self.rw_head.sequence].steps.len();

        if self.rw_head.step == 0 {
            self.rw_head.step = len - 1;
        } else {
            self.rw_head.step -= 1;
        }
        // info!("rec head sequence = {}, {len}", self.play_head.sequence);
    }

    pub fn get_name(&self) -> String {
        if let Some(name) = self.sequences[self.rw_head.sequence].human_name.clone() {
            name
        } else {
            format!("{}", self.rw_head.sequence)
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

        if at <= self.rw_head.sequence {
            self.rw_head.sequence -= 1;
        }

        if at <= self.view_head.sequence {
            self.view_head.sequence -= 1;
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
        self.sequences[self.rw_head.sequence].clone()
    }

    pub fn set_rec_head_seq(&mut self, seq: i64) {
        self.rw_head.sequence = (seq % self.sequences.len() as i64) as usize;
    }

    pub fn set_sequence(&mut self, sequence: usize) {
        if sequence < self.sequences.len() {
            self.rw_head.set_sequence(sequence);
        } else {
            error!("attempted to set record head to {sequence}, but that sequence doesn't exist.");
        }
    }

    pub fn set_rec_channel(&mut self, channel: SequenceChannel) {
        self.rw_head.set_channel(channel);
    }
}

impl MidiControlled for SequencerIntake {
    fn midi_input(&mut self, message: &MidiMessage) {
        #[cfg(feature = "pyo3")]
        self.synth.midi_input(message);

        if let MidiMessage::ControlChange(_channel, ControlEvent { control, value: _ }) = message {
            match control {
                115 => {
                    self.rw_head.step = if self.rw_head.step > 0 {
                        self.rw_head.step - 1
                    } else {
                        self.sequences[self.rw_head.sequence].steps.len() - 1
                    };
                }
                116 => {
                    self.rw_head.step += 1;
                    self.rw_head.step %= self.sequences[self.rw_head.sequence].steps.len();
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
            MidiMessage::PitchBend(_channel, _lsb, _msb) => return,
            _ => {
                return;
            }
        };

        let step = if on_enter {
            &mut self.sequences[self.rw_head.clone()].on_enter
        } else {
            &mut self.sequences[self.rw_head.clone()].on_exit
        };

        let step_filter_f = |(_rec_ch, rec_msg)| {
            // if ch == rec_ch {
            match (rec_msg, msg.clone()) {
                (StepCmd::Play { note: n1, vel: _ }, StepCmd::Play { note: n2, vel: _ })
                    if n1 == n2 =>
                {
                    Some(())
                }
                (StepCmd::CC { code: c1, value: _ }, StepCmd::CC { code: c2, value: _ })
                    if c1 == c2 =>
                {
                    Some(())
                }
                (StepCmd::Stop { note: n1 }, StepCmd::Stop { note: n2 }) if n1 == n2 => Some(()),
                _ => None,
            }
            // } else {
            //     None
            // }
        };

        let step_not_contains = step[self.rw_head.channel]
            .clone()
            .into_iter()
            .filter_map(step_filter_f)
            .collect::<Vec<()>>()
            .len()
            == 0;

        if step_not_contains {
            step[self.rw_head.channel].insert((self.rw_head.channel.into(), msg));
        } else {
            // info!("rm'ing a message");
            // info!("rm'ing a message from on_enter {on_enter}");
            // info!("number of messages before = {}", step.len());
            step[self.rw_head.channel].retain(|msg| step_filter_f(msg.clone()).is_none());
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
        for (midi_chan, midi_mesg) in midi_s {
            // let instrument =
            // if midi.0 == 0 {
            //     synth.get_engine()
            // } else
            // if let Some(synth_type) = synth_types.get((midi.0 - 1) as usize) {
            //     synth.engines.index_mut(*synth_type as usize)
            // } else {
            //     continue;
            // };

            match midi_mesg {
                StepCmd::Play { note, vel } => {
                    playing.insert((midi_chan, note));

                    synth
                        .get_channel_engine(midi_chan.into())
                        .engine
                        .play(note, vel)
                }
                StepCmd::Stop { note } => {
                    playing.remove(&(midi_chan, note));

                    synth.get_channel_engine(midi_chan.into()).engine.stop(note)
                }
                StepCmd::CC { code: _, value: _ } => {}
            }
        }
    };

    let mut send_channel = |synth: &mut Synth, seq_chan: PerChannelMidi| {
        for midi_channel in seq_chan.by_channel() {
            send_midi(synth, midi_channel)
        }
    };

    let mut play_step = |last_on_exit: PerChannelMidi| {
        // info!("beat");
        let mut seq = seq.lock().unwrap();
        // info!("after sequence lock");
        let step = seq.sequences[seq.rw_head].clone();
        send_channel(&mut seq.synth, last_on_exit);

        send_channel(&mut seq.synth, step.on_enter);
        step.on_exit.clone()
        // .by_channel()
        // .into_iter()
        // .flatten()
        // .collect()
    };
    let inc_step = || {
        // info!("beat");
        let mut seq = seq.lock().unwrap();
        seq.rw_head.step += 1;
        seq.rw_head.step %= seq.sequences[seq.rw_head.sequence].steps.len();
    };

    // let mut last_on_exit = play_step(HashSet::default());
    let mut last_on_exit = play_step(PerChannelMidi::default());
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
    seq.rw_head.step = 0;
    playing.into_iter().for_each(|(ch, note)| {
        let synth = &mut seq.synth;

        synth.get_channel_engine(ch.into()).engine.stop(note);

        // if ch == 0 {
        //     synth.get_engine().stop(note);
        // } else if let Some(synth_type) = synth_types.get((ch - 1) as usize) {
        //     synth.engines.index_mut(*synth_type as usize).stop(note);
        // }
    })
}
