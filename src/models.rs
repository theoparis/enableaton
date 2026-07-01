use crate::audio::SampleType;
use crate::theme::STEPS;

#[derive(Clone, Debug)]
pub struct Pattern {
    pub start: usize,
    pub len: usize,
    pub steps: [bool; STEPS],
}

#[derive(Clone, Debug)]
pub struct DrumPad {
    pub name: String,
    pub sample: SampleType,
    pub color: u32,
    pub steps: [bool; STEPS],
    pub muted: bool,
}

/// What plays when a track fires.
///
/// `Sampler` fires once per active pattern step. `DrumRack` owns its own per-pad
/// step grid and fires each pad independently on every step inside an active pattern.
#[derive(Clone, Debug)]
pub enum Instrument {
    Sampler(SampleType),
    DrumRack(Vec<DrumPad>),
}

impl Instrument {
    pub fn short_label(&self) -> &'static str {
        match self {
            Instrument::Sampler(s) => s.short_label(),
            Instrument::DrumRack(_) => "DR",
        }
    }

    pub fn badge_color(&self) -> u32 {
        match self {
            Instrument::Sampler(s) => s.badge_color(),
            Instrument::DrumRack(_) => 0x2a1535,
        }
    }

    /// Preview trigger (e.g. clicking the track name): fires the first non-muted pad.
    pub fn trigger(&self, audio: &crate::audio::AudioEngine) {
        match self {
            Instrument::Sampler(s) => audio.trigger(*s),
            Instrument::DrumRack(pads) => {
                if let Some(pad) = pads.iter().find(|p| !p.muted) {
                    audio.trigger(pad.sample);
                }
            }
        }
    }

    /// Called every step inside an active arrangement pattern.
    /// For Sampler, fires only when `pat_steps[step]` is set.
    /// For DrumRack, ignores `pat_steps` and checks each pad's own grid.
    pub fn trigger_step(
        &self,
        step: usize,
        pat_steps: &[bool; STEPS],
        audio: &crate::audio::AudioEngine,
    ) {
        match self {
            Instrument::Sampler(s) => {
                if pat_steps[step] {
                    audio.trigger(*s);
                }
            }
            Instrument::DrumRack(pads) => {
                for pad in pads {
                    if !pad.muted && pad.steps[step] {
                        audio.trigger(pad.sample);
                    }
                }
            }
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
