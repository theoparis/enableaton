use crate::audio::SampleType;
use crate::theme::STEPS;

#[derive(Clone, Debug)]
pub struct Pattern {
    pub start: usize,
    pub len: usize,
    pub steps: [bool; STEPS],
}

/// What plays when a track's step fires.
///
/// `Sampler` is a single one-shot voice. Future variants (e.g. `DrumRack`)
/// would carry their own pad grid and trigger multiple voices per step.
#[derive(Clone, Debug)]
pub enum Instrument {
    Sampler(SampleType),
}

impl Instrument {
    pub fn short_label(&self) -> &'static str {
        match self {
            Instrument::Sampler(s) => s.short_label(),
        }
    }

    pub fn badge_color(&self) -> u32 {
        match self {
            Instrument::Sampler(s) => s.badge_color(),
        }
    }

    pub fn trigger(&self, audio: &crate::audio::AudioEngine) {
        match self {
            Instrument::Sampler(s) => audio.trigger(*s),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub name: String,
    pub color: u32,
    pub muted: bool,
    pub patterns: Vec<Pattern>,
    pub instrument: Instrument,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DragKind {
    Move,
    Resize,
}

#[derive(Clone, Debug)]
pub struct DragState {
    pub track_idx: usize,
    pub pat_idx: usize,
    pub kind: DragKind,
    pub orig_start: usize,
    pub orig_len: usize,
    pub mouse_start_x: f32,
}
