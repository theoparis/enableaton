use gpui::{AppContext, Context, Entity, ScrollHandle, Task, point, px};

use crate::ui::core::slider::Slider;
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use std::sync::Arc;

use crate::audio::{AudioEngine, SampleType, WaveShape, WavetableSynth};
use crate::models::{
    Clip, DragKind, DragState, DrumPad, Instrument, Note, ParamDrag, PatternData, PianoRollResize,
    Track,
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
    /// Song (playlist) vs. pattern playback mode toggle in the toolbar.
    pub song_mode: bool,
    pub show_browser: bool,
    pub show_channel_rack: bool,
    /// Lower docked window: piano roll / channel settings.
    pub show_piano_roll: bool,
    pub show_mixer: bool,
    /// Window-space x of the playlist grid's left edge (recorded at paint).
    pub playlist_origin_x: Rc<Cell<f32>>,
    /// Window-space x of the piano-roll note grid's left edge (recorded at paint).
    pub piano_roll_origin_x: Rc<Cell<f32>>,
    pub audio: Option<AudioEngine>,
    pub _tick_task: Option<Task<()>>,
    pub selected_track: Option<usize>,
    pub bpm_drag_origin: Option<(f32, u32)>,
    pub pattern_scroll: ScrollHandle,
    pub piano_roll_scroll: ScrollHandle,
    pub piano_roll_zoom: f32,
    pub piano_roll_resize: Option<PianoRollResize>,
    /// (track, pattern) the piano roll was last centered on; a change
    /// re-centers the view on that pattern's notes.
    pub piano_roll_focus: Option<(usize, usize)>,
    pub master_vol: f32,
    pub master_slider: Entity<Slider>,
    /// Show the device (plugin settings) panel instead of the piano roll.
    pub show_device: bool,
    pub param_drag: Option<ParamDrag>,
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

        let bass_synth = Arc::new(
            WavetableSynth::new(WaveShape::Saw, "BS", 0x1a2a6a)
                .with_adsr(0.005, 0.1, 0.6, 0.15)
                .with_note_len(0.3)
                .with_amplitude(0.6),
        );
        let lead_synth = Arc::new(
            WavetableSynth::new(WaveShape::Saw, "SY", 0x143d35)
                .with_adsr(0.005, 0.08, 0.5, 0.1)
                .with_note_len(0.25)
                .with_amplitude(0.5),
        );
        let pad_synth = Arc::new(
            WavetableSynth::new(WaveShape::Sine, "PD", 0x3a2050)
                .with_adsr(0.3, 0.2, 0.8, 0.5)
                .with_note_len(1.0)
                .with_amplitude(0.4),
        );

        // 16 steps @ 1/16 = 4 beats of pattern-relative notes.
        fn steps_to_notes(steps: &[bool; STEPS], pitch: u8) -> Vec<Note> {
            (0..STEPS)
                .filter(|&s| steps[s])
                .map(|s| Note {
                    pitch,
                    start: s as f32 * 0.25,
                    duration: 0.25,
                })
                .collect()
        }

        fn note_pattern(name: &str, steps: &[bool; STEPS], pitch: u8) -> PatternData {
            PatternData {
                name: name.into(),
                len: 4,
                lanes: vec![],
                notes: steps_to_notes(steps, pitch),
            }
        }

        fn clips_at(starts: &[usize]) -> Vec<Clip> {
            starts
                .iter()
                .map(|&s| Clip {
                    pattern: 0,
                    start: s,
                })
                .collect()
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
            song_mode: true,
            show_browser: true,
            show_channel_rack: true,
            show_piano_roll: true,
            show_mixer: true,
            playlist_origin_x: Rc::new(Cell::new(0.0)),
            piano_roll_origin_x: Rc::new(Cell::new(0.0)),
            audio,
            _tick_task: None,
            // Open on the lead synth so the piano roll has content in view.
            selected_track: Some(2),
            bpm_drag_origin: None,
            pattern_scroll: ScrollHandle::default(),
            piano_roll_scroll: ScrollHandle::default(),
            piano_roll_zoom: 4.0,
            piano_roll_resize: None,
            piano_roll_focus: None,
            show_device: false,
            param_drag: None,
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
                    patterns: vec![PatternData {
                        name: "P1".into(),
                        len: 4,
                        lanes: vec![
                            kick_steps,
                            snare_steps,
                            hihat_steps,
                            openhat_steps,
                            clap_steps,
                        ],
                        notes: vec![],
                    }],
                    // Same pattern placed back-to-back = the loop.
                    clips: clips_at(&[0, 4, 8, 12, 16, 20, 24, 28]),
                    active_pattern: 0,
                    instrument: Instrument::DrumRack(vec![
                        DrumPad {
                            name: "Kick".into(),
                            sample: SampleType::Kick,
                            color: TRACK_COLORS[0],
                            muted: false,
                        },
                        DrumPad {
                            name: "Snare".into(),
                            sample: SampleType::Snare,
                            color: TRACK_COLORS[1],
                            muted: false,
                        },
                        DrumPad {
                            name: "Hi-Hat".into(),
                            sample: SampleType::HiHat,
                            color: TRACK_COLORS[2],
                            muted: false,
                        },
                        DrumPad {
                            name: "Open Hat".into(),
                            sample: SampleType::OpenHat,
                            color: TRACK_COLORS[3],
                            muted: false,
                        },
                        DrumPad {
                            name: "Clap".into(),
                            sample: SampleType::Clap,
                            color: TRACK_COLORS[4],
                            muted: false,
                        },
                    ]),
                },
                Track {
                    name: "Bass".into(),
                    color: TRACK_COLORS[5],
                    muted: false,
                    patterns: vec![note_pattern("P1", &bass_steps, 36)],
                    clips: clips_at(&[2, 10, 18, 22]),
                    active_pattern: 0,
                    instrument: Instrument::Plugin(bass_synth),
                },
                Track {
                    name: "Synth Lead".into(),
                    color: TRACK_COLORS[6],
                    muted: false,
                    patterns: vec![note_pattern("P1", &synth_steps, 65)],
                    clips: clips_at(&[4, 8, 16, 24]),
                    active_pattern: 0,
                    instrument: Instrument::Plugin(lead_synth),
                },
                Track {
                    name: "Pad".into(),
                    color: TRACK_COLORS[0],
                    muted: false,
                    patterns: vec![note_pattern("P1", &pad_steps, 60)],
                    clips: clips_at(&[0, 8, 16, 24]),
                    active_pattern: 0,
                    instrument: Instrument::Plugin(pad_synth),
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
                            // ── Piano roll: fire clip notes starting in [prev_pos, play_pos)
                            if !wrapped {
                                let cur = this.play_pos;
                                for t in &this.tracks {
                                    if t.muted || !t.instrument.uses_piano_roll() {
                                        continue;
                                    }
                                    for clip in &t.clips {
                                        let pat = &t.patterns[clip.pattern];
                                        let cs = clip.start as f32;
                                        if prev_pos >= cs + pat.len as f32 || cur <= cs {
                                            continue;
                                        }
                                        for note in &pat.notes {
                                            if note.start >= pat.len as f32 {
                                                continue;
                                            }
                                            let fire = cs + note.start;
                                            if fire >= prev_pos && fire < cur {
                                                t.instrument.trigger_note(note.pitch, audio);
                                            }
                                        }
                                    }
                                }
                            }

                            // ── Drum lanes: fire on step boundaries ─────────────────────
                            let steps_per_beat = 4_usize;
                            let global_step = (this.play_pos * steps_per_beat as f32) as i32;
                            if global_step != this.last_step {
                                this.last_step = global_step;
                                for t in &this.tracks {
                                    if t.muted {
                                        continue;
                                    }
                                    if let Some(c_idx) = t.clip_at(this.play_pos) {
                                        let clip = &t.clips[c_idx];
                                        let pat = &t.patterns[clip.pattern];
                                        let elapsed = this.play_pos - clip.start as f32;
                                        let step = (elapsed * steps_per_beat as f32).floor()
                                            as usize
                                            % pat.steps_in_loop();
                                        t.instrument.trigger_step(step, &pat.lanes, audio);
                                    }
                                }
                            }
                        }

                        this.last_play_pos = this.play_pos;

                        // Auto-scroll the playlist (gpui scroll offsets are ≤ 0).
                        let ph_x = this.play_pos * BEAT_PX;
                        let target_offset = (ph_x - 150.0).max(0.0);
                        this.pattern_scroll
                            .set_offset(point(px(-target_offset), px(0.)));

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

    /// Beat under a window-space x coordinate in the playlist grid.
    pub fn playlist_beat_at(&self, window_x: f32) -> f32 {
        let scroll_x = f32::from(self.pattern_scroll.offset().x);
        ((window_x - self.playlist_origin_x.get() - scroll_x) / BEAT_PX).max(0.0)
    }

    pub fn scrub_to(&mut self, window_x: f32) {
        let beat = self.playlist_beat_at(window_x);
        let total = (self.total_bars * self.beats_per_bar) as f32;
        self.play_pos = beat.min(total - 0.001);
    }

    pub fn apply_drag(&mut self, mouse_x: f32) {
        let Some(ref drag) = self.drag.clone() else {
            return;
        };
        let delta = ((mouse_x - drag.mouse_start_x) / BEAT_PX).round() as isize;
        let total = (self.total_bars * self.beats_per_bar) as isize;
        let track = &mut self.tracks[drag.track_idx];
        let clip = track.clips[drag.clip_idx];
        match drag.kind {
            DragKind::Move => {
                track.clips[drag.clip_idx].start = (drag.orig_start as isize + delta)
                    .max(0)
                    .min(total - drag.orig_len as isize)
                    as usize;
            }
            DragKind::Resize => {
                // Resizing a clip resizes the pattern — every instance follows.
                track.patterns[clip.pattern].len = (drag.orig_len as isize + delta).max(1) as usize;
            }
        }
    }

    /// Insert the track's active pattern on the timeline at `beat` (skipped
    /// when it would overlap an existing clip).
    pub fn insert_clip(&mut self, track_idx: usize, beat: usize) {
        let track = &mut self.tracks[track_idx];
        if track.patterns.is_empty() {
            track.add_pattern();
            track.active_pattern = 0;
        }
        let pattern = track.active_pattern.min(track.patterns.len() - 1);
        let len = track.patterns[pattern].len;
        let total = self.total_bars * self.beats_per_bar;
        let start = beat.min(total.saturating_sub(len));
        if !track.clip_overlaps(start, len) {
            track.clips.push(Clip { pattern, start });
        }
    }

    pub fn add_track(&mut self) {
        let idx = self.tracks.len() % TRACK_COLORS.len();
        let color = TRACK_COLORS[idx];
        self.tracks.push(Track {
            name: format!("Track {}", self.tracks.len() + 1),
            color,
            muted: false,
            instrument: Instrument::Plugin(Arc::new(WavetableSynth::new(
                WaveShape::Saw,
                "WT",
                color,
            ))),
            patterns: vec![PatternData::new("P1", 4, 0)],
            clips: vec![],
            active_pattern: 0,
        });
    }
}
