use crate::audio::SampleType;
use crate::theme::STEPS;

#[derive(Clone, Debug)]
pub struct Note {
    pub pitch: u8,     // MIDI note number, C4 = 60
    pub start: f32,    // absolute timeline position in beats
    pub duration: f32, // in beats
}

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

    /// Preview trigger (no pitch). Used for drum rack pads and sampler track clicks.
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

    /// Trigger with a specific MIDI pitch (piano roll note playback).
    pub fn trigger_note(&self, pitch: u8, audio: &crate::audio::AudioEngine) {
        match self {
            Instrument::Sampler(s) => audio.trigger_pitched(*s, Some(pitch)),
            Instrument::DrumRack(_) => {} // drum rack uses trigger_step
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
    /// Piano-roll note data (used when instrument is Sampler; empty for DrumRack).
    pub notes: Vec<Note>,
}

#[derive(Clone, Debug)]
pub struct PianoRollResize {
    pub track_idx: usize,
    /// Identifies the note by (pitch, start) so the index stays valid after re-renders.
    pub note_pitch: u8,
    pub note_start: f32,
    pub orig_duration: f32,
    pub drag_start_x: f32,
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
