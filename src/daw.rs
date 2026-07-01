use gpui::{AppContext, Context, Entity, ScrollHandle, Task, point, px};

use crate::ui::core::slider::Slider;
use std::time::{Duration, Instant};

use crate::audio::{AudioEngine, SampleType};
use crate::models::{
    DragKind, DragState, DrumPad, Instrument, Note, Pattern, PianoRollResize, Track,
};
use crate::theme::*;

pub struct Daw {
    pub playing: bool,
    pub recording: bool,
    pub bpm: u32,
    pub play_pos: f32,
    pub last_play_pos: f32,
    pub last_step: i32,
    pub tracks: Vec<Track>,
    pub beats_per_bar: usize,
    pub total_bars: usize,
    pub drag: Option<DragState>,
    pub scrubbing: bool,
    pub show_beat_editor: bool,
    pub audio: Option<AudioEngine>,
    pub _tick_task: Option<Task<()>>,
    pub selected_track: Option<usize>,
    pub bpm_drag_origin: Option<(f32, u32)>,
    pub pattern_scroll: ScrollHandle,
    pub piano_roll_scroll: ScrollHandle,
    pub piano_roll_zoom: f32,
    pub piano_roll_resize: Option<PianoRollResize>,
    pub master_vol: f32,
    pub master_slider: Entity<Slider>,
}

impl Daw {
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        let audio = AudioEngine::start();
        if audio.is_none() {
            eprintln!("warning: no audio device found, sequencer will be silent");
        }

        let kick_steps = [
            true, false, false, false, true, false, false, false, true, false, false, false, true,
            false, false, false,
        ];
        let snare_steps = [
            false, false, false, false, true, false, false, false, false, false, false, false,
            true, false, false, false,
        ];
        let hihat_steps = [
            true, false, true, false, true, false, true, false, true, false, true, false, true,
            false, true, false,
        ];
        let openhat_steps = [
            false, false, false, false, false, false, true, false, false, false, false, false,
            false, false, true, false,
        ];
        let clap_steps = [
            false, false, false, false, true, false, false, true, false, false, false, false, true,
            false, false, false,
        ];
        let bass_steps = [
            true, false, false, true, false, false, true, false, true, false, false, false, true,
            false, false, true,
        ];
        let synth_steps = [
            true, false, false, false, false, false, true, false, false, true, false, false, true,
            false, false, false,
        ];
        let pad_steps = [
            true, false, false, false, false, false, false, false, true, false, false, false,
            false, false, false, false,
        ];

        fn steps_to_notes(
            patterns: &[(usize, usize)],
            steps: &[bool; STEPS],
            pitch: u8,
        ) -> Vec<Note> {
            let mut notes = Vec::new();
            for &(start, _len) in patterns {
                for s in 0..STEPS {
                    if steps[s] {
                        notes.push(Note {
                            pitch,
                            start: start as f32 + s as f32 * 0.25,
                            duration: 0.25,
                        });
                    }
                }
            }
            notes
        }

        Self {
            playing: false,
            recording: false,
            bpm: 128,
            play_pos: 0.0,
            last_play_pos: 0.0,
            last_step: -1,
            beats_per_bar: 4,
            total_bars: 32,
            drag: None,
            scrubbing: false,
            show_beat_editor: true,
            audio,
            _tick_task: None,
            selected_track: None,
            bpm_drag_origin: None,
            pattern_scroll: ScrollHandle::default(),
            piano_roll_scroll: {
                let h = ScrollHandle::default();
                // Start scrolled to show C3–F4; offset is negative (more negative = further down).
                h.set_offset(point(px(0.0), px(-270.0)));
                h
            },
            piano_roll_zoom: 2.0,
            piano_roll_resize: None,
            master_vol: 0.72,
            master_slider: {
                let daw_entity = cx.entity();
                cx.new(|_| {
                    Slider::new(0.72).on_change(move |val, app| {
                        app.update_entity(&daw_entity, |daw, cx| {
                            daw.master_vol = val;
                            if let Some(audio) = &daw.audio {
                                audio.set_master_vol(val);
                            }
                            cx.notify();
                        });
                    })
                })
            },
            tracks: vec![
                Track {
                    name: "Drums".into(),
                    color: TRACK_COLORS[0],
                    muted: false,
                    patterns: vec![Pattern {
                        start: 0,
                        len: 32,
                        steps: [true; STEPS],
                    }],
                    notes: vec![],
                    instrument: Instrument::DrumRack(vec![
                        DrumPad {
                            name: "Kick".into(),
                            sample: SampleType::Kick,
                            color: TRACK_COLORS[0],
                            steps: kick_steps,
                            muted: false,
                        },
                        DrumPad {
                            name: "Snare".into(),
                            sample: SampleType::Snare,
                            color: TRACK_COLORS[1],
                            steps: snare_steps,
                            muted: false,
                        },
                        DrumPad {
                            name: "Hi-Hat".into(),
                            sample: SampleType::HiHat,
                            color: TRACK_COLORS[2],
                            steps: hihat_steps,
                            muted: false,
                        },
                        DrumPad {
                            name: "Open Hat".into(),
                            sample: SampleType::OpenHat,
                            color: TRACK_COLORS[3],
                            steps: openhat_steps,
                            muted: false,
                        },
                        DrumPad {
                            name: "Clap".into(),
                            sample: SampleType::Clap,
                            color: TRACK_COLORS[4],
                            steps: clap_steps,
                            muted: false,
                        },
                    ]),
                },
                Track {
                    name: "Bass".into(),
                    color: TRACK_COLORS[5],
                    muted: false,
                    // Arrangement blocks kept for song-editor display; steps unused (notes drive playback).
                    patterns: [(2, 6), (10, 4), (18, 10)]
                        .iter()
                        .map(|&(s, l)| Pattern {
                            start: s,
                            len: l,
                            steps: [false; STEPS],
                        })
                        .collect(),
                    // C3 = 48: speed × 2 over C2 base, sounds an octave up — tight bass register.
                    notes: steps_to_notes(&[(2, 6), (10, 4), (18, 10)], &bass_steps, 48),
                    instrument: Instrument::Sampler(SampleType::Bass),
                },
                Track {
                    name: "Synth Lead".into(),
                    color: TRACK_COLORS[6],
                    muted: false,
                    patterns: [(4, 8), (16, 12)]
                        .iter()
                        .map(|&(s, l)| Pattern {
                            start: s,
                            len: l,
                            steps: [false; STEPS],
                        })
                        .collect(),
                    // F4 = 65: one semitone above base (E4=64), nearly unchanged timbre.
                    notes: steps_to_notes(&[(4, 8), (16, 12)], &synth_steps, 65),
                    instrument: Instrument::Sampler(SampleType::SynthLead),
                },
                Track {
                    name: "Pad".into(),
                    color: TRACK_COLORS[0],
                    muted: false,
                    patterns: [(0usize, 28usize)]
                        .iter()
                        .map(|&(s, l)| Pattern {
                            start: s,
                            len: l,
                            steps: [false; STEPS],
                        })
                        .collect(),
                    // C4 = 60: a fourth above G3 base, brightens the pad slightly.
                    notes: steps_to_notes(&[(0, 28)], &pad_steps, 60),
                    instrument: Instrument::Sampler(SampleType::Pad),
                },
            ],
        }
    }

    pub fn start_tick(&mut self, cx: &mut Context<Self>) {
        let mut last_tick = Instant::now();
        let task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_spawn(async {
                    std::thread::sleep(Duration::from_millis(8));
                })
                .await;

                let keep_going = this
                    .update(cx, |this, cx| {
                        if !this.playing {
                            return false;
                        }
                        let now = Instant::now();
                        let elapsed_secs = (now - last_tick).as_secs_f32();
                        last_tick = now;
                        let beats_per_tick = this.bpm as f32 / 60.0 * elapsed_secs;
                        let prev_pos = this.play_pos;
                        this.play_pos += beats_per_tick;
                        let total = (this.total_bars * this.beats_per_bar) as f32;
                        let wrapped = this.play_pos >= total;
                        if wrapped {
                            this.play_pos = 0.0;
                        }

                        if let Some(audio) = &this.audio {
                            // ── Piano roll: fire notes that start in [prev_pos, play_pos) ──
                            if !wrapped {
                                for t in &this.tracks {
                                    if t.muted {
                                        continue;
                                    }
                                    if matches!(t.instrument, Instrument::Sampler(_)) {
                                        for note in &t.notes {
                                            if note.start >= prev_pos && note.start < this.play_pos
                                            {
                                                t.instrument.trigger_note(note.pitch, audio);
                                            }
                                        }
                                    }
                                }
                            }

                            // ── Drum rack: fire on step boundaries ──────────────────────
                            let steps_per_beat = 4_usize;
                            let global_step = (this.play_pos * steps_per_beat as f32) as i32;
                            if global_step != this.last_step {
                                this.last_step = global_step;
                                for t in &this.tracks {
                                    if t.muted {
                                        continue;
                                    }
                                    if let Some(pat) = t.patterns.iter().find(|pat| {
                                        let start = pat.start as f32;
                                        let end = (pat.start + pat.len) as f32;
                                        this.play_pos >= start && this.play_pos < end
                                    }) {
                                        let elapsed = this.play_pos - pat.start as f32;
                                        let step = (elapsed * steps_per_beat as f32).floor()
                                            as usize
                                            % STEPS;
                                        t.instrument.trigger_step(step, &pat.steps, audio);
                                    }
                                }
                            }
                        }

                        this.last_play_pos = this.play_pos;

                        // Auto-scroll song editor.
                        let ph_x = this.play_pos * BEAT_PX;
                        let target_offset = (ph_x - 150.0).max(0.0);
                        this.pattern_scroll
                            .set_offset(point(px(target_offset), px(0.)));

                        cx.notify();
                        true
                    })
                    .unwrap_or(false);

                if !keep_going {
                    break;
                }
            }
        });
        self._tick_task = Some(task);
    }

    pub fn stop_tick(&mut self) {
        self._tick_task = None;
    }

    pub fn toggle_play(&mut self, cx: &mut Context<Self>) {
        self.playing = !self.playing;
        if self.playing {
            self.start_tick(cx);
        } else {
            self.stop_tick();
        }
    }

    pub fn scrub_to(&mut self, window_x: f32) {
        let scroll_x = f32::from(self.pattern_scroll.offset().x);
        let beat = ((window_x - HEADER_W + scroll_x) / BEAT_PX).max(0.0);
        let total = (self.total_bars * self.beats_per_bar) as f32;
        self.play_pos = beat.min(total - 0.001);
    }

    pub fn apply_drag(&mut self, mouse_x: f32) {
        let Some(ref drag) = self.drag.clone() else {
            return;
        };
        let delta = ((mouse_x - drag.mouse_start_x) / BEAT_PX).round() as isize;
        let total = (self.total_bars * self.beats_per_bar) as isize;
        let pat = &mut self.tracks[drag.track_idx].patterns[drag.pat_idx];
        match drag.kind {
            DragKind::Move => {
                pat.start = (drag.orig_start as isize + delta)
                    .max(0)
                    .min(total - drag.orig_len as isize) as usize;
            }
            DragKind::Resize => {
                pat.len = (drag.orig_len as isize + delta).max(1) as usize;
            }
        }
    }

    pub fn add_track(&mut self) {
        let idx = self.tracks.len() % TRACK_COLORS.len();
        self.tracks.push(Track {
            name: format!("Track {}", self.tracks.len() + 1),
            color: TRACK_COLORS[idx],
            muted: false,
            instrument: Instrument::Sampler(SampleType::HiHat),
            patterns: vec![],
            notes: vec![],
        });
    }

    pub fn active_pattern_idx(track: &crate::models::Track, play_pos: f32) -> Option<usize> {
        track.patterns.iter().position(|pat| {
            let start = pat.start as f32;
            let end = (pat.start + pat.len) as f32;
            play_pos >= start && play_pos < end
        })
    }
}
