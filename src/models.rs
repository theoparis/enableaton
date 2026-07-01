use std::sync::Arc;

use crate::audio::{InstrumentPlugin, SampleType};
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
/// `Sampler` fires a percussion sample. `DrumRack` owns its own per-pad step
/// grid. `Plugin` delegates synthesis to an `InstrumentPlugin` implementation.
pub enum Instrument {
    Sampler(SampleType),
    DrumRack(Vec<DrumPad>),
    Plugin(Arc<dyn InstrumentPlugin>),
}

impl Clone for Instrument {
    fn clone(&self) -> Self {
        match self {
            Instrument::Sampler(s) => Instrument::Sampler(*s),
            Instrument::DrumRack(pads) => Instrument::DrumRack(pads.clone()),
            Instrument::Plugin(p) => Instrument::Plugin(Arc::clone(p)),
        }
    }
}

impl std::fmt::Debug for Instrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instrument::Sampler(s) => f.debug_tuple("Sampler").field(s).finish(),
            Instrument::DrumRack(pads) => f.debug_tuple("DrumRack").field(&pads.len()).finish(),
            Instrument::Plugin(p) => write!(f, "Plugin({})", p.short_label()),
        }
    }
}

impl Instrument {
    pub fn short_label(&self) -> String {
        match self {
            Instrument::Sampler(s) => s.short_label().to_string(),
            Instrument::DrumRack(_) => "DR".to_string(),
            Instrument::Plugin(p) => p.short_label().to_string(),
        }
    }

    pub fn badge_color(&self) -> u32 {
        match self {
            Instrument::Sampler(s) => s.badge_color(),
            Instrument::DrumRack(_) => 0x2a1535,
            Instrument::Plugin(p) => p.badge_color(),
        }
    }

    /// Returns true for instrument types that use piano-roll note data for playback.
    pub fn uses_piano_roll(&self) -> bool {
        matches!(self, Instrument::Sampler(_) | Instrument::Plugin(_))
    }

    /// Preview trigger (no pitch). Used for track header clicks.
    pub fn trigger(&self, audio: &crate::audio::AudioEngine) {
        match self {
            Instrument::Sampler(s) => audio.trigger(*s),
            Instrument::DrumRack(pads) => {
                if let Some(pad) = pads.iter().find(|p| !p.muted) {
                    audio.trigger(pad.sample);
                }
            }
            Instrument::Plugin(p) => audio.trigger_plugin(p.as_ref(), None),
        }
    }

    /// Trigger with a specific MIDI pitch (piano roll note playback).
    pub fn trigger_note(&self, pitch: u8, audio: &crate::audio::AudioEngine) {
        match self {
            Instrument::Sampler(s) => audio.trigger(*s),
            Instrument::DrumRack(_) => {}
            Instrument::Plugin(p) => audio.trigger_plugin(p.as_ref(), Some(pitch)),
        }
    }

    /// Called every step inside an active arrangement pattern.
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
            Instrument::Plugin(p) => {
                if pat_steps[step] {
                    audio.trigger_plugin(p.as_ref(), Some(60));
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
    /// Piano-roll note data (used by Sampler and Plugin tracks; empty for DrumRack).
    pub notes: Vec<Note>,
}

#[derive(Clone, Debug)]
pub struct PianoRollResize {
    pub track_idx: usize,
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
