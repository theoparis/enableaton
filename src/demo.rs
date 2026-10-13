//! Built-in demo song: A minor, Am – F – C – G, 128 BPM, 32 bars.
//!
//! Arrangement (bars):
//!   0–3   intro      pad + hats
//!   4–7   build      + bass, kick/off-beat hats
//!   8–15  verse      full drums, bass, arp (lead joins at bar 12)
//!   16–19 breakdown  pad, bass, high lead melody, riser drums at the end
//!   20–27 drop       everything, fill on the last bar
//!   28–31 outro      pad + hats

use std::sync::Arc;

use crate::audio::{SampleType, WaveShape, WavetableSynth};
use crate::models::{Clip, DrumPad, Instrument, Note, PatternData, Track};
use crate::theme::*;

/// One chord per bar, 4 bars per pattern.
const PROGRESSION: [[u8; 3]; 4] = [
    [57, 60, 64], // Am
    [53, 57, 60], // F
    [55, 60, 64], // C/G
    [55, 59, 62], // G
];
/// Bass roots for the same four bars.
const BASS_ROOTS: [u8; 4] = [45, 41, 48, 43];

const PHRASE_BEATS: usize = 16;

fn steps(on: &[usize]) -> [bool; STEPS] {
    let mut lane = [false; STEPS];
    for &s in on {
        lane[s] = true;
    }
    lane
}

fn note(pitch: u8, start: f32, duration: f32) -> Note {
    Note {
        pitch,
        start,
        duration,
    }
}

fn clips(pattern: usize, starts: &[usize]) -> Vec<Clip> {
    starts
        .iter()
        .map(|&start| Clip { pattern, start })
        .collect()
}

fn note_pattern(name: &str, notes: Vec<Note>) -> PatternData {
    PatternData {
        name: name.into(),
        len: PHRASE_BEATS,
        lanes: vec![],
        notes,
    }
}

fn drum_pattern(name: &str, lanes: [[bool; STEPS]; 5]) -> PatternData {
    PatternData {
        name: name.into(),
        len: 4,
        lanes: lanes.to_vec(),
        notes: vec![],
    }
}

fn drums() -> Track {
    let groove = drum_pattern(
        "Groove",
        [
            steps(&[0, 4, 8, 12]),                  // kick
            steps(&[4, 12]),                        // snare
            steps(&[0, 2, 4, 6, 7, 8, 10, 12, 15]), // closed hat
            steps(&[14]),                           // open hat
            steps(&[4, 12]),                        // clap
        ],
    );
    let intro = drum_pattern(
        "Intro",
        [
            steps(&[]),
            steps(&[]),
            steps(&[0, 2, 4, 6, 8, 10, 12, 14]),
            steps(&[]),
            steps(&[]),
        ],
    );
    let fill = drum_pattern(
        "Fill",
        [
            steps(&[0, 4, 8, 12]),
            steps(&[8, 10, 12, 13, 14, 15]),
            steps(&[0, 2, 4, 6]),
            steps(&[]),
            steps(&[12]),
        ],
    );
    let build = drum_pattern(
        "Build",
        [
            steps(&[0, 4, 8, 12]),
            steps(&[]),
            steps(&[2, 6, 10, 14]),
            steps(&[]),
            steps(&[]),
        ],
    );
    const GROOVE: usize = 0;
    const INTRO: usize = 1;
    const FILL: usize = 2;
    const BUILD: usize = 3;

    let mut c = Vec::new();
    c.extend(clips(INTRO, &[0, 4, 8, 12]));
    c.extend(clips(BUILD, &[16, 20, 24, 28]));
    c.extend(clips(GROOVE, &[32, 36, 40, 44, 48, 52, 56]));
    c.extend(clips(FILL, &[60]));
    c.extend(clips(BUILD, &[76]));
    c.extend(clips(GROOVE, &[80, 84, 88, 92, 96, 100, 104]));
    c.extend(clips(FILL, &[108]));
    c.extend(clips(INTRO, &[112, 116, 120, 124]));

    let pad = |name: &str, sample, i: usize| DrumPad {
        name: name.into(),
        sample,
        color: TRACK_COLORS[i],
        muted: false,
    };

    Track {
        name: "Drums".into(),
        color: TRACK_COLORS[0],
        muted: false,
        patterns: vec![groove, intro, fill, build],
        clips: c,
        active_pattern: 0,
        instrument: Instrument::DrumRack(vec![
            pad("Kick", SampleType::Kick, 0),
            pad("Snare", SampleType::Snare, 1),
            pad("Hi-Hat", SampleType::HiHat, 2),
            pad("Open Hat", SampleType::OpenHat, 3),
            pad("Clap", SampleType::Clap, 4),
        ]),
    }
}

fn bass() -> Track {
    // Per-bar rhythm: (start beat, duration, octave offset).
    const RHYTHM: [(f32, f32, u8); 6] = [
        (0.0, 0.5, 0),
        (0.75, 0.25, 12),
        (1.5, 0.5, 0),
        (2.0, 0.5, 0),
        (2.75, 0.25, 12),
        (3.5, 0.5, 0),
    ];
    let notes = BASS_ROOTS
        .iter()
        .enumerate()
        .flat_map(|(bar, &root)| {
            RHYTHM.iter().map(move |&(start, dur, oct)| {
                note(root + oct, bar as f32 * 4.0 + start, dur * 0.9)
            })
        })
        .collect();

    let synth = Arc::new(
        WavetableSynth::new(WaveShape::Saw, "BS", 0x1a2a6a)
            .with_adsr(0.004, 0.12, 0.65, 0.08)
            .with_cutoff(650.0)
            .with_amplitude(0.42),
    );
    Track {
        name: "Bass".into(),
        color: TRACK_COLORS[5],
        muted: false,
        patterns: vec![note_pattern("Groove", notes)],
        clips: clips(0, &[16, 32, 48, 64, 80, 96]),
        active_pattern: 0,
        instrument: Instrument::Plugin(synth),
    }
}

fn arp() -> Track {
    // Index into [root, third, fifth, root + octave].
    const SEQ: [usize; 16] = [0, 1, 2, 3, 2, 3, 1, 2, 0, 1, 2, 3, 2, 3, 1, 3];
    let notes = PROGRESSION
        .iter()
        .enumerate()
        .flat_map(|(bar, chord)| {
            let tones = [chord[0] + 12, chord[1] + 12, chord[2] + 12, chord[0] + 24];
            SEQ.iter()
                .enumerate()
                .map(move |(step, &i)| note(tones[i], bar as f32 * 4.0 + step as f32 * 0.25, 0.2))
        })
        .collect();

    let synth = Arc::new(
        WavetableSynth::new(WaveShape::Saw, "AR", 0x143d35)
            .with_adsr(0.003, 0.12, 0.2, 0.1)
            .with_detune(14.0)
            .with_cutoff(3200.0)
            .with_amplitude(0.24),
    );
    Track {
        name: "Arp".into(),
        color: TRACK_COLORS[2],
        muted: false,
        patterns: vec![note_pattern("Arp", notes)],
        clips: clips(0, &[16, 32, 48, 80, 96]),
        active_pattern: 0,
        instrument: Instrument::Plugin(synth),
    }
}

fn lead() -> Track {
    // (start beat, duration, pitch)
    const VERSE: &[(f32, f32, u8)] = &[
        // Am
        (0.0, 1.5, 76),
        (1.5, 0.5, 74),
        (2.0, 1.0, 72),
        (3.0, 1.0, 76),
        // F
        (4.0, 1.5, 77),
        (5.5, 0.5, 76),
        (6.0, 1.0, 72),
        (7.0, 1.0, 69),
        // C
        (8.0, 1.5, 79),
        (9.5, 0.5, 76),
        (10.0, 1.0, 72),
        (11.0, 0.5, 76),
        (11.5, 0.5, 79),
        // G
        (12.0, 1.0, 74),
        (13.0, 0.5, 71),
        (13.5, 0.5, 74),
        (14.0, 2.0, 79),
    ];
    const BREAK: &[(f32, f32, u8)] = &[
        (0.0, 3.0, 81),
        (3.0, 1.0, 79),
        (4.0, 3.0, 77),
        (7.0, 1.0, 76),
        (8.0, 3.0, 79),
        (11.0, 1.0, 76),
        (12.0, 2.0, 74),
        (14.0, 2.0, 71),
    ];
    let to_notes = |src: &[(f32, f32, u8)]| {
        src.iter()
            .map(|&(start, dur, pitch)| note(pitch, start, dur * 0.95))
            .collect::<Vec<_>>()
    };

    let synth = Arc::new(
        WavetableSynth::new(WaveShape::Saw, "LD", 0x4a2a14)
            .with_adsr(0.02, 0.1, 0.7, 0.25)
            .with_detune(18.0)
            .with_cutoff(4500.0)
            .with_amplitude(0.24),
    );
    // Gentle vibrato.
    {
        use crate::audio::{InstrumentPlugin, P_LFO_DEPTH, P_LFO_RATE, P_LFO_TARGET};
        synth.set_param(P_LFO_TARGET, 1.0);
        synth.set_param(P_LFO_RATE, 5.5);
        synth.set_param(P_LFO_DEPTH, 0.12);
    }

    let mut c = clips(0, &[48, 80, 96]);
    c.extend(clips(1, &[64]));
    Track {
        name: "Lead".into(),
        color: TRACK_COLORS[6],
        muted: false,
        patterns: vec![
            note_pattern("Verse", to_notes(VERSE)),
            note_pattern("Break", to_notes(BREAK)),
        ],
        clips: c,
        active_pattern: 0,
        instrument: Instrument::Plugin(synth),
    }
}

fn pad() -> Track {
    let notes = PROGRESSION
        .iter()
        .enumerate()
        .flat_map(|(bar, chord)| chord.iter().map(move |&p| note(p, bar as f32 * 4.0, 3.9)))
        .collect();

    let synth = Arc::new(
        WavetableSynth::new(WaveShape::Saw, "PD", 0x3a2050)
            .with_adsr(0.45, 0.3, 0.8, 0.6)
            .with_detune(22.0)
            .with_cutoff(1400.0)
            .with_amplitude(0.11),
    );
    Track {
        name: "Pad".into(),
        color: TRACK_COLORS[1],
        muted: false,
        patterns: vec![note_pattern("Chords", notes)],
        clips: clips(0, &[0, 16, 32, 48, 64, 80, 96, 112]),
        active_pattern: 0,
        instrument: Instrument::Plugin(synth),
    }
}

pub fn tracks() -> Vec<Track> {
    vec![drums(), bass(), arp(), lead(), pad()]
}
