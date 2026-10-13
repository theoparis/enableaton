use gpui::{AppContext, Bounds, Context, Entity, Pixels, ScrollHandle, Task, point, px};

use crate::ui::core::slider::Slider;
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use std::sync::Arc;

use crate::audio::{AudioEngine, WaveShape, WavetableSynth};
use crate::models::{
    Clip, DragKind, DragState, Instrument, Mix, PaneEdge, ParamDrag, PatternData, PianoRollResize,
    Track, UiDrag,
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
    /// Per-frame playback signal; only overlays and the toolbar observe it.
    pub ticker: Entity<crate::ui::Ticker>,
    /// ~20 Hz signal for the mixer's fake meters.
    pub meter_clock: Entity<crate::ui::Ticker>,
    pub panes: Option<crate::ui::Panes>,
    pub tick_n: u32,
    /// Window-space bounds of the playlist / piano-roll grids (recorded at
    /// paint) so the playhead overlays can draw over cached panes.
    pub playlist_grid: Rc<Cell<Bounds<Pixels>>>,
    pub piano_roll_grid: Rc<Cell<Bounds<Pixels>>>,
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
    /// Per-track mixer state, parallel to `tracks`.
    pub mix: Vec<Mix>,
    pub ui_drag: Option<UiDrag>,
    /// Index into the toolbar menu bar of the open dropdown.
    pub open_menu: Option<usize>,
    /// Transient message shown on the toolbar hint line.
    pub status: Option<String>,
    pub show_playlist: bool,
    pub browser_w: f32,
    pub mid_w: f32,
    pub rack_h: f32,
    pub mixer_h: f32,
    /// Browser tree expansion: current project, generators, patterns.
    pub expanded: [bool; 3],
}

impl Daw {
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        let audio = AudioEngine::start();
        if audio.is_none() {
            eprintln!("warning: no audio device found, sequencer will be silent");
        }

        let tracks = crate::demo::tracks();
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
            ticker: cx.new(|_| crate::ui::Ticker),
            meter_clock: cx.new(|_| crate::ui::Ticker),
            panes: None,
            tick_n: 0,
            playlist_grid: Rc::new(Cell::new(Bounds::default())),
            piano_roll_grid: Rc::new(Cell::new(Bounds::default())),
            audio,
            _tick_task: None,
            // Open on the arp so the piano roll has content in view.
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
            mix: vec![Mix::default(); tracks.len()],
            tracks,
            ui_drag: None,
            open_menu: None,
            status: None,
            show_playlist: true,
            browser_w: BROWSER_W,
            mid_w: MID_COL_W,
            rack_h: RACK_H,
            mixer_h: MIXER_H,
            expanded: [true, true, false],
        }
    }

    pub fn start_tick(&mut self, cx: &mut Context<Self>) {
        let mut last_tick = Instant::now();
        let task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(16))
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
                        let total = this.loop_len();
                        let wrapped = this.play_pos >= total;
                        if wrapped {
                            this.play_pos = 0.0;
                        }
                        let step_changed =
                            wrapped || (this.play_pos * 4.0) as i32 != this.last_step;

                        if let Some(audio) = &this.audio {
                            // ── Piano roll: fire clip notes starting in [prev_pos, play_pos)
                            if !wrapped {
                                let cur = this.play_pos;
                                for (ti, t) in this.tracks.iter().enumerate() {
                                    if t.muted || !t.instrument.uses_piano_roll() {
                                        continue;
                                    }
                                    let gain = this.gain(ti);
                                    for (pi, cs) in this.instances(ti) {
                                        let pat = &t.patterns[pi];
                                        if prev_pos >= cs + pat.len as f32 || cur <= cs {
                                            continue;
                                        }
                                        for note in &pat.notes {
                                            if note.start >= pat.len as f32 {
                                                continue;
                                            }
                                            let fire = cs + note.start;
                                            if fire >= prev_pos && fire < cur {
                                                let hold = note.duration * 60.0 / this.bpm as f32;
                                                t.instrument.trigger_note(
                                                    note.pitch,
                                                    Some(hold),
                                                    audio,
                                                    gain,
                                                );
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
                                for (ti, t) in this.tracks.iter().enumerate() {
                                    if t.muted {
                                        continue;
                                    }
                                    if let Some((pi, cs)) = this.pattern_at(ti, this.play_pos) {
                                        let pat = &t.patterns[pi];
                                        let elapsed = this.play_pos - cs;
                                        let step = (elapsed * steps_per_beat as f32).floor()
                                            as usize
                                            % pat.steps_in_loop();
                                        t.instrument.trigger_step(
                                            step,
                                            &pat.lanes,
                                            audio,
                                            this.gain(ti),
                                        );
                                    }
                                }
                            }
                        }

                        this.last_play_pos = this.play_pos;

                        // Re-center the view only on a transition: the playhead
                        // crossing the right edge, or the loop wrapping. A
                        // standing "off-screen" test would fight manual
                        // scrolling every frame (gpui scroll offsets are ≤ 0).
                        let ph_x = this.play_pos * BEAT_PX;
                        let prev_x = prev_pos * BEAT_PX;
                        let view_left = -f32::from(this.pattern_scroll.offset().x);
                        let edge =
                            view_left + f32::from(this.pattern_scroll.bounds().size.width) - 40.0;
                        let scrolled = wrapped || (prev_x <= edge && ph_x > edge);
                        if scrolled {
                            let target_offset = (ph_x - 150.0).max(0.0);
                            this.pattern_scroll
                                .set_offset(point(px(-target_offset), px(0.)));
                        }

                        // Panes are cached views observing `Daw`, so notify it
                        // only when something they draw can have changed: a
                        // 16th-step boundary (step cursor, active clip), a loop
                        // wrap, or a playlist scroll jump. The playhead itself
                        // is drawn by overlays fed by the per-frame ticker.
                        if step_changed || scrolled {
                            cx.notify();
                        }
                        this.tick_n = this.tick_n.wrapping_add(1);
                        let ticker = this.ticker.clone();
                        ticker.update(cx, |_, cx| cx.notify());
                        if this.tick_n % 3 == 0 {
                            let clock = this.meter_clock.clone();
                            clock.update(cx, |_, cx| cx.notify());
                        }
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

    /// Length of the transport loop in beats: the whole song, or in pattern
    /// mode the selected track's active pattern.
    pub fn loop_len(&self) -> f32 {
        if !self.song_mode
            && let Some(t) = self.selected_track.and_then(|i| self.tracks.get(i))
            && let Some(p) = t.patterns.get(t.active_pattern)
        {
            return p.len.max(1) as f32;
        }
        (self.total_bars * self.beats_per_bar) as f32
    }

    /// `(pattern, start beat)` instances of track `t` that the transport
    /// plays. Pattern mode plays only the selected track's active pattern.
    pub fn instances(&self, t: usize) -> Vec<(usize, f32)> {
        let track = &self.tracks[t];
        if self.song_mode {
            track
                .clips
                .iter()
                .map(|c| (c.pattern, c.start as f32))
                .collect()
        } else if self.selected_track == Some(t) && !track.patterns.is_empty() {
            vec![(track.active_pattern.min(track.patterns.len() - 1), 0.0)]
        } else {
            Vec::new()
        }
    }

    /// Pattern instance of track `t` under beat `pos`, as `(pattern, start)`.
    pub fn pattern_at(&self, t: usize, pos: f32) -> Option<(usize, f32)> {
        let track = &self.tracks[t];
        self.instances(t)
            .into_iter()
            .find(|&(p, s)| pos >= s && pos < s + track.patterns[p].len as f32)
    }

    pub fn gain(&self, t: usize) -> f32 {
        self.mix.get(t).copied().unwrap_or_default().gain()
    }

    /// Preview-play track `t`'s instrument at its mixer gain.
    pub fn preview_track(&self, t: usize) {
        if let Some(audio) = &self.audio {
            self.tracks[t].instrument.trigger(audio, self.gain(t));
        }
    }

    /// Apply an active pane/fader/pan drag for the pointer at `(x, y)`.
    pub fn apply_ui_drag(&mut self, x: f32, y: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.ui_drag else { return };
        match drag {
            UiDrag::Pane { edge, start, orig } => match edge {
                PaneEdge::Browser => self.browser_w = (orig + x - start).clamp(120.0, 420.0),
                PaneEdge::MidColumn => self.mid_w = (orig + x - start).clamp(280.0, 900.0),
                PaneEdge::Rack => self.rack_h = (orig + y - start).clamp(80.0, 520.0),
                // Dragging up grows the mixer.
                PaneEdge::Mixer => self.mixer_h = (orig + start - y).clamp(120.0, 520.0),
            },
            UiDrag::Fader {
                track,
                start_y,
                orig,
            } => {
                let v = (orig + (start_y - y) / 120.0).clamp(0.0, 1.0);
                match track {
                    Some(t) => self.mix[t].volume = v,
                    None => {
                        self.master_vol = v;
                        if let Some(a) = &self.audio {
                            a.set_master_vol(v);
                        }
                        self.master_slider.update(cx, |s, cx| {
                            s.set_value(v);
                            cx.notify();
                        });
                    }
                }
            }
            UiDrag::Pan {
                track,
                start_x,
                orig,
            } => {
                self.mix[track].pan = (orig + (x - start_x) / 60.0).clamp(-1.0, 1.0);
            }
        }
        cx.notify();
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
        self.mix.push(Mix::default());
    }
}
