use crate::audio::SampleType;
use crate::theme::STEPS;

#[derive(Clone, Debug)]
pub struct Pattern {
    pub start: usize,
    pub len: usize,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub name: String,
    pub color: u32,
    pub muted: bool,
    pub patterns: Vec<Pattern>,
    pub steps: [bool; STEPS],
    pub sample: SampleType,
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
