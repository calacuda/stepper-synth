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

#[pyclass(module = "stepper_synth_backend", get_all)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum PythonCmd {
    SetSynthParam(SynthParam),
    Exit(),
}

pub type State = SynthParam;

// #[pyclass(module = "stepper_synth_backend")]
// #[derive(Debug, Clone)]
// pub struct State {
//
// }
