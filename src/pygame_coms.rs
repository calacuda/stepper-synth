use pyo3::pyclass;

#[pyclass(module = "stepper_synth_backend", eq, get_all)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum SynthParam {
    Atk(f32),
    Dcy(f32),
    Sus(f32),
    Rel(f32),
    FilterCutoff(f32),
    FilterRes(f32),
    DelayVol(f32),
    DelayTime(f32),
    SpeakerSpinSpeed(f32),
    PitchBend(f32),
}

/// a MIDI Note
// pub type Note = u8;
// /// a collection of all the known phrases.
// pub type Phrases = [Option<Phrase>; 256];
/// an index into a list of all known type T
// pub type Index = usize;

#[pyclass(module = "stepper_synth_backend", get_all, eq, eq_int)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

#[pyclass(module = "stepper_synth_backend", get_all, eq, eq_int)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

#[pyclass(module = "stepper_synth_backend", get_all)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum PythonCmd {
    // SetSynthParam(SynthParam),
    SetGuiParam { param: GuiParam, set_to: f32 },
    SetKnob { knob: Knob, set_to: f32 },
    Exit(),
}

pub type State = SynthParam;

// #[pyclass(module = "stepper_synth_backend")]
// #[derive(Debug, Clone)]
// pub struct State {
//
// }
