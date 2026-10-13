use std::sync::Arc;

use crate::audio::{InstrumentPlugin, SampleType};
use crate::theme::STEPS;

#[derive(Clone, Debug)]
pub struct Note {
    pub pitch: u8,     // MIDI note number, C4 = 60
    pub start: f32,    // position in beats, relative to the pattern start
    pub duration: f32, // in beats
}

/// Shared pattern content. Clips on the timeline reference a pattern by
/// index; placing the same pattern twice plays the same content (editing the
/// pattern updates every clip that uses it).
#[derive(Clone, Debug)]
pub struct PatternData {
    pub name: String,
    /// Length in beats.
    pub len: usize,
    /// Step lanes (drum racks: one lane per pad; samplers: single lane).
    pub lanes: Vec<[bool; STEPS]>,
    /// Piano-roll notes (synth/sampler tracks), relative to pattern start.
    pub notes: Vec<Note>,
}

impl PatternData {
    pub fn new(name: impl Into<String>, len: usize, n_lanes: usize) -> Self {
        Self {
            name: name.into(),
            len,
            lanes: vec![[false; STEPS]; n_lanes],
            notes: Vec::new(),
        }
    }

    /// Number of drum steps in one pattern pass (16ths, capped at grid size).
    pub fn steps_in_loop(&self) -> usize {
        (self.len * 4).clamp(1, STEPS)
    }
}

/// An instance of a pattern placed on the timeline.
#[derive(Clone, Copy, Debug)]
pub struct Clip {
    pub pattern: usize,
    /// Timeline position in beats.
    pub start: usize,
}

#[derive(Clone, Debug)]
pub struct DrumPad {
    pub name: String,
    pub sample: SampleType,
    pub color: u32,
    pub muted: bool,
}

/// What plays when a track fires.
///
/// `Sampler` fires a percussion sample. `DrumRack` plays per-pad step lanes
/// from the active pattern. `Plugin` delegates synthesis to an
/// `InstrumentPlugin` implementation.
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

    /// Number of step lanes patterns on this instrument should carry.
    pub fn n_lanes(&self) -> usize {
        match self {
            Instrument::Sampler(_) => 1,
            Instrument::DrumRack(pads) => pads.len(),
            Instrument::Plugin(_) => 0,
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
            Instrument::Plugin(p) => audio.trigger_plugin(p.as_ref(), None, None),
        }
    }
    /// Trigger with a specific MIDI pitch. `hold_secs` is the note length for
    /// sequenced playback; `None` for live previews.
    pub fn trigger_note(
        &self,
        pitch: u8,
        hold_secs: Option<f32>,
        audio: &crate::audio::AudioEngine,
    ) {
        match self {
            Instrument::Sampler(s) => audio.trigger(*s),
            Instrument::DrumRack(_) => {}
            Instrument::Plugin(p) => audio.trigger_plugin(p.as_ref(), Some(pitch), hold_secs),
        }
    }

    /// Called every step while a clip is under the playhead.
    pub fn trigger_step(
        &self,
        step: usize,
        lanes: &[[bool; STEPS]],
        audio: &crate::audio::AudioEngine,
    ) {
        match self {
            Instrument::Sampler(s) => {
                if lanes.first().is_some_and(|lane| lane[step]) {
                    audio.trigger(*s);
                }
            }
            Instrument::DrumRack(pads) => {
                for (pad, lane) in pads.iter().zip(lanes) {
                    if !pad.muted && lane[step] {
                        audio.trigger(pad.sample);
                    }
                }
            }
            Instrument::Plugin(_) => {}
        }
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub name: String,
    pub color: u32,
    pub muted: bool,
    pub instrument: Instrument,
    /// Shared pattern pool for this track.
    pub patterns: Vec<PatternData>,
    /// Pattern instances placed on the timeline.
    pub clips: Vec<Clip>,
    /// Pattern currently being edited / inserted on click.
    pub active_pattern: usize,
}

impl Track {
    /// Index of the clip under `pos`, if any.
    pub fn clip_at(&self, pos: f32) -> Option<usize> {
        self.clips.iter().position(|clip| {
            let start = clip.start as f32;
            let end = start + self.patterns[clip.pattern].len as f32;
            pos >= start && pos < end
        })
    }

    /// Create a new empty pattern sized for this instrument and return its index.
    pub fn add_pattern(&mut self) -> usize {
        let idx = self.patterns.len();
        self.patterns.push(PatternData::new(
            format!("P{}", idx + 1),
            4,
            self.instrument.n_lanes(),
        ));
        idx
    }

    /// True if a clip of `len` beats starting at `start` would overlap an
    /// existing clip.
    pub fn clip_overlaps(&self, start: usize, len: usize) -> bool {
        self.clips.iter().any(|c| {
            let c_end = c.start + self.patterns[c.pattern].len;
            start < c_end && start + len > c.start
        })
    }
}

/// Vertical drag on a device-panel parameter cell.
#[derive(Clone, Debug)]
pub struct ParamDrag {
    pub track_idx: usize,
    pub param_idx: usize,
    pub start_y: f32,
    pub start_val: f32,
}

#[derive(Clone, Debug)]
pub struct PianoRollResize {
    pub track_idx: usize,
    pub pattern_idx: usize,
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
    pub clip_idx: usize,
    pub kind: DragKind,
    pub orig_start: usize,
    pub orig_len: usize,
    pub mouse_start_x: f32,
}
