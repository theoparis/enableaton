use gpui::{
    ClickEvent, Context, FontWeight, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Render, SharedString, Task, Window, div, prelude::*, px, rgb,
};
use std::time::Duration;

use crate::audio::{AudioEngine, SampleType};
use crate::models::{DragKind, DragState, Pattern, Track};
use crate::theme::*;

pub struct Daw {
    pub playing: bool,
    pub recording: bool,
    pub bpm: u32,
    pub play_pos: f32,  // fractional beats
    pub last_step: i32, // last triggered 16th-note step (-1 = none)
    pub tracks: Vec<Track>,
    pub beats_per_bar: usize,
    pub total_bars: usize,
    pub drag: Option<DragState>,
    pub scrubbing: bool,
    pub show_beat_editor: bool,
    pub audio: Option<AudioEngine>,
    pub _tick_task: Option<Task<()>>,
}

impl Daw {
    pub fn new() -> Self {
        let audio = AudioEngine::start();
        if audio.is_none() {
            eprintln!("warning: no audio device found, sequencer will be silent");
        }

        Self {
            playing: false,
            recording: false,
            bpm: 128,
            play_pos: 0.0,
            last_step: -1,
            beats_per_bar: 4,
            total_bars: 32,
            drag: None,
            scrubbing: false,
            show_beat_editor: true,
            audio,
            _tick_task: None,
            tracks: vec![
                Track {
                    name: "Kick".into(),
                    color: TRACK_COLORS[0],
                    muted: false,
                    sample: SampleType::Kick,
                    patterns: vec![
                        Pattern { start: 0, len: 4 },
                        Pattern { start: 4, len: 4 },
                        Pattern { start: 12, len: 8 },
                        Pattern { start: 24, len: 8 },
                    ],
                    //  1 . . . | 2 . . . | 3 . . . | 4 . . .
                    steps: [
                        true, false, false, false, true, false, false, false, true, false, false,
                        false, true, false, false, false,
                    ],
                },
                Track {
                    name: "Snare".into(),
                    color: TRACK_COLORS[1],
                    muted: false,
                    sample: SampleType::Snare,
                    patterns: vec![Pattern { start: 0, len: 8 }, Pattern { start: 16, len: 8 }],
                    steps: [
                        false, false, false, false, true, false, false, false, false, false, false,
                        false, true, false, false, false,
                    ],
                },
                Track {
                    name: "Hi-Hat".into(),
                    color: TRACK_COLORS[2],
                    muted: false,
                    sample: SampleType::HiHat,
                    patterns: vec![Pattern { start: 0, len: 16 }, Pattern { start: 20, len: 4 }],
                    // 8th-note hi-hats
                    steps: [
                        true, false, true, false, true, false, true, false, true, false, true,
                        false, true, false, true, false,
                    ],
                },
                Track {
                    name: "Open Hat".into(),
                    color: TRACK_COLORS[3],
                    muted: false,
                    sample: SampleType::OpenHat,
                    patterns: vec![Pattern { start: 4, len: 4 }, Pattern { start: 20, len: 4 }],
                    steps: [
                        false, false, false, false, false, false, true, false, false, false, false,
                        false, false, false, true, false,
                    ],
                },
                Track {
                    name: "Clap".into(),
                    color: TRACK_COLORS[4],
                    muted: false,
                    sample: SampleType::Clap,
                    patterns: vec![Pattern { start: 2, len: 6 }, Pattern { start: 18, len: 10 }],
                    steps: [
                        false, false, false, false, true, false, false, true, false, false, false,
                        false, true, false, false, false,
                    ],
                },
                Track {
                    name: "Bass".into(),
                    color: TRACK_COLORS[5],
                    muted: false,
                    sample: SampleType::Bass,
                    patterns: vec![
                        Pattern { start: 2, len: 6 },
                        Pattern { start: 10, len: 4 },
                        Pattern { start: 18, len: 10 },
                    ],
                    steps: [
                        true, false, false, true, false, false, true, false, true, false, false,
                        false, true, false, false, true,
                    ],
                },
                Track {
                    name: "Synth Lead".into(),
                    color: TRACK_COLORS[6],
                    muted: false,
                    sample: SampleType::SynthLead,
                    patterns: vec![Pattern { start: 4, len: 8 }, Pattern { start: 16, len: 12 }],
                    steps: [
                        true, false, false, false, false, false, true, false, false, true, false,
                        false, true, false, false, false,
                    ],
                },
                Track {
                    name: "Pad".into(),
                    color: TRACK_COLORS[0],
                    muted: false,
                    sample: SampleType::Pad,
                    patterns: vec![Pattern { start: 0, len: 28 }],
                    steps: [
                        true, false, false, false, false, false, false, false, true, false, false,
                        false, false, false, false, false,
                    ],
                },
            ],
        }
    }

    pub fn start_tick(&mut self, cx: &mut Context<Self>) {
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
                        let beats_per_tick = this.bpm as f32 / 60_000.0 * 8.0;
                        this.play_pos += beats_per_tick;
                        let total = (this.total_bars * this.beats_per_bar) as f32;
                        if this.play_pos >= total {
                            this.play_pos = 0.0;
                        }

                        // Trigger beat sequencer at 16th-note boundaries.
                        let steps_per_beat = 4_usize;
                        let global_step = (this.play_pos * steps_per_beat as f32) as i32;
                        if global_step != this.last_step {
                            this.last_step = global_step;
                            for t in &this.tracks {
                                if t.muted {
                                    continue;
                                }
                                // Check if there is an active pattern covering the current play_pos
                                if let Some(pat) = t.patterns.iter().find(|pat| {
                                    let start = pat.start as f32;
                                    let end = (pat.start + pat.len) as f32;
                                    this.play_pos >= start && this.play_pos < end
                                }) {
                                    // Calculate relative step index within the pattern
                                    let elapsed_beats = this.play_pos - pat.start as f32;
                                    let pattern_step =
                                        (elapsed_beats * steps_per_beat as f32).floor() as usize
                                            % STEPS;
                                    if t.steps[pattern_step] {
                                        if let Some(audio) = &this.audio {
                                            audio.trigger(t.sample);
                                        }
                                    }
                                }
                            }
                        }

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
        let beat = ((window_x - HEADER_W) / BEAT_PX).max(0.0);
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
            sample: SampleType::HiHat,
            patterns: vec![],
            steps: [false; STEPS],
        });
    }
}

// ── Render ────────────────────────────────────────────────────────────────────

impl Render for Daw {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let total_beats = self.total_bars * self.beats_per_bar;
        let total_w = BEAT_PX * total_beats as f32;
        let total_bars = self.total_bars;
        let bpb = self.beats_per_bar;
        let bpm = self.bpm;
        let playing = self.playing;
        let recording = self.recording;
        let play_pos = self.play_pos;
        let n_tracks = self.tracks.len();
        let show_beat = self.show_beat_editor;
        let current_step = if playing {
            (play_pos * 4.0) as usize % STEPS
        } else {
            usize::MAX
        };

        // position display
        let bar = (play_pos / bpb as f32).floor() as usize + 1;
        let beat = (play_pos % bpb as f32).floor() as usize + 1;

        // ── Transport ─────────────────────────────────────────────────────────

        let transport = div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px_3()
            .h(px(TRANSPORT_H))
            .flex_none()
            .bg(rgb(SURFACE))
            .border_b_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(ACCENT))
                    .mr_3()
                    .child("WAVE"),
            )
            // rewind
            .child(transport_btn(
                "⏮",
                "rew",
                false,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.play_pos = 0.0;
                    cx.notify();
                }),
            ))
            // play
            .child(
                div()
                    .id("btn-play")
                    .px_3()
                    .py_1()
                    .rounded_sm()
                    .bg(rgb(if playing { ACCENT } else { SURFACE2 }))
                    .text_color(rgb(if playing { 0x000000 } else { TEXT }))
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(ACCENT_DIM)))
                    .on_click(cx.listener(|this, _: &ClickEvent, _w, cx| {
                        this.toggle_play(cx);
                        cx.notify();
                    }))
                    .child(if playing { "⏸" } else { "▶" }),
            )
            // stop
            .child(transport_btn(
                "⏹",
                "stop",
                false,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.playing = false;
                    this.play_pos = 0.0;
                    this.stop_tick();
                    cx.notify();
                }),
            ))
            // record
            .child(
                div()
                    .id("btn-rec")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .bg(rgb(if recording { RED_MUTED } else { SURFACE2 }))
                    .text_color(rgb(0xdd4444))
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(RED_MUTED)))
                    .on_click(cx.listener(|this, _: &ClickEvent, _w, cx| {
                        this.recording = !this.recording;
                        cx.notify();
                    }))
                    .child("⏺"),
            )
            .child(div().w_px().h_6().bg(rgb(BORDER)).mx_2())
            // BPM
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(dim("BPM"))
                    .child(
                        div()
                            .px_2()
                            .py_px()
                            .bg(rgb(BG))
                            .rounded_sm()
                            .border_1()
                            .border_color(rgb(BORDER))
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            .child(format!("{bpm}")),
                    ),
            )
            // position
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .ml_2()
                    .child(dim("POS"))
                    .child(
                        div()
                            .px_2()
                            .py_px()
                            .bg(rgb(BG))
                            .rounded_sm()
                            .border_1()
                            .border_color(rgb(BORDER))
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(TEXT))
                            .child(format!("{bar} : {beat} : 00")),
                    ),
            )
            .child(div().flex_1())
            // beat editor toggle
            .child(
                div()
                    .id("beat-toggle")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .bg(rgb(if show_beat { ACCENT_DIM } else { SURFACE2 }))
                    .text_color(rgb(if show_beat { ACCENT } else { TEXT_DIM }))
                    .text_xs()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(BORDER)))
                    .on_click(cx.listener(|this, _: &ClickEvent, _w, cx| {
                        this.show_beat_editor = !this.show_beat_editor;
                        cx.notify();
                    }))
                    .child("BEAT"),
            )
            .child(div().w_px().h_6().bg(rgb(BORDER)).mx_2())
            // master vol
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(dim("MASTER"))
                    .child(
                        div()
                            .w_24()
                            .h_2()
                            .bg(rgb(BORDER))
                            .rounded_full()
                            .overflow_hidden()
                            .child(div().w(px(72.)).h_full().bg(rgb(ACCENT_DIM)).rounded_full()),
                    ),
            );

        // ── Track headers ─────────────────────────────────────────────────────

        let mut headers = div()
            .flex_none()
            .w(px(HEADER_W))
            .flex()
            .flex_col()
            .bg(rgb(SURFACE))
            .border_r_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .h(px(RULER_H))
                    .flex()
                    .items_center()
                    .px_2()
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .child(dim("SONG EDITOR")),
            );

        for idx in 0..n_tracks {
            let t = &self.tracks[idx];
            let color = t.color;
            let muted = t.muted;
            let name = t.name.clone();
            headers = headers.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .h(px(TRACK_H))
                    .bg(rgb(SURFACE))
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .child(div().w(px(3.)).h_6().rounded_full().bg(rgb(color)))
                    .child(
                        div()
                            .id(SharedString::from(format!("preview-{idx}")))
                            .flex_1()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(if muted { TEXT_DIM } else { TEXT }))
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _: &ClickEvent, _w, _cx| {
                                if let Some(audio) = &this.audio {
                                    audio.trigger(this.tracks[idx].sample);
                                }
                            }))
                            .child(name),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("mute-{idx}")))
                            .px_1()
                            .py_px()
                            .rounded_sm()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(if muted { ACCENT } else { TEXT_DIM }))
                            .cursor_pointer()
                            .hover(|s| s.text_color(rgb(ACCENT)))
                            .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                                this.tracks[idx].muted ^= true;
                                cx.notify();
                            }))
                            .child("M"),
                    ),
            );
        }

        headers = headers.child(
            div()
                .id("add-track")
                .flex()
                .items_center()
                .justify_center()
                .h_8()
                .mx_2()
                .my_1()
                .rounded_sm()
                .bg(rgb(SURFACE2))
                .border_1()
                .border_color(rgb(BORDER))
                .text_xs()
                .text_color(rgb(TEXT_DIM))
                .cursor_pointer()
                .hover(|s| s.bg(rgb(BORDER)).text_color(rgb(TEXT)))
                .on_click(cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.add_track();
                    cx.notify();
                }))
                .child("+ Add Track"),
        );

        // ── Pattern area ──────────────────────────────────────────────────────

        let ph_x = play_pos * BEAT_PX;

        // Ruler
        let mut ruler = div()
            .id("ruler")
            .relative()
            .flex_none()
            .h(px(RULER_H))
            .w(px(total_w))
            .bg(rgb(SURFACE))
            .border_b_1()
            .border_color(rgb(BORDER))
            .cursor_col_resize()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, ev: &MouseDownEvent, _w, cx| {
                    this.scrubbing = true;
                    this.scrub_to(f32::from(ev.position.x));
                    cx.notify();
                }),
            );

        for bar in 0..=total_bars {
            let x = bar as f32 * bpb as f32 * BEAT_PX;
            ruler = ruler
                .child(
                    div()
                        .absolute()
                        .left(px(x))
                        .top_0()
                        .w_px()
                        .h_full()
                        .bg(rgb(BORDER)),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(x + 2.))
                        .top(px(4.))
                        .text_xs()
                        .text_color(rgb(TEXT_DIM))
                        .child(format!("{}", bar + 1)),
                );
        }
        ruler = ruler.child(
            div()
                .absolute()
                .left(px(ph_x))
                .top_0()
                .w(px(2.))
                .h_full()
                .bg(rgb(PLAYHEAD))
                .opacity(0.9),
        );

        let mut pattern_col = div().flex().flex_col().w(px(total_w)).child(ruler);

        for t_idx in 0..n_tracks {
            let track = &self.tracks[t_idx];
            let color = track.color;
            let muted = track.muted;

            let mut row = div()
                .relative()
                .flex_none()
                .h(px(TRACK_H))
                .w(px(total_w))
                .bg(rgb(if muted { 0x111620 } else { BG }))
                .border_b_1()
                .border_color(rgb(BORDER));

            for bar in 0..=total_bars {
                let x = bar as f32 * bpb as f32 * BEAT_PX;
                row = row.child(
                    div()
                        .absolute()
                        .left(px(x))
                        .top_0()
                        .w_px()
                        .h_full()
                        .bg(rgb(BORDER))
                        .opacity(0.5),
                );
            }

            for p_idx in 0..track.patterns.len() {
                let pat = &track.patterns[p_idx];
                let x = pat.start as f32 * BEAT_PX;
                let w = (pat.len as f32 * BEAT_PX - 1.0).max(4.0);
                let blk_color = if muted { 0x1a2233u32 } else { color };
                let resize_id = SharedString::from(format!("rsz-{t_idx}-{p_idx}"));
                let move_id = SharedString::from(format!("pat-{t_idx}-{p_idx}"));

                let block = div()
                    .absolute()
                    .left(px(x))
                    .top(px(3.))
                    .w(px(w))
                    .h(px(TRACK_H - 6.))
                    .bg(rgb(blk_color))
                    .rounded_sm()
                    .border_l_2()
                    .border_color(rgb(if muted { TEXT_DIM } else { ACCENT }))
                    .overflow_hidden()
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .right_0()
                            .h(px(1.))
                            .bg(rgb(0xffffff))
                            .opacity(0.1),
                    )
                    .child(
                        div()
                            .id(resize_id)
                            .absolute()
                            .right_0()
                            .top_0()
                            .w(px(6.))
                            .h_full()
                            .cursor_col_resize()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, ev: &MouseDownEvent, _w, cx| {
                                    let pat = &this.tracks[t_idx].patterns[p_idx];
                                    this.drag = Some(DragState {
                                        track_idx: t_idx,
                                        pat_idx: p_idx,
                                        kind: DragKind::Resize,
                                        orig_start: pat.start,
                                        orig_len: pat.len,
                                        mouse_start_x: f32::from(ev.position.x),
                                    });
                                    cx.notify();
                                }),
                            ),
                    );

                let block_with_move = div()
                    .id(move_id)
                    .absolute()
                    .left(px(x))
                    .top(px(3.))
                    .w(px(w))
                    .h(px(TRACK_H - 6.))
                    .cursor_grab()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, ev: &MouseDownEvent, _w, cx| {
                            let mx = f32::from(ev.position.x);
                            let pat = &this.tracks[t_idx].patterns[p_idx];
                            let rel = mx - HEADER_W - pat.start as f32 * BEAT_PX;
                            if rel < pat.len as f32 * BEAT_PX - 6.0 {
                                this.drag = Some(DragState {
                                    track_idx: t_idx,
                                    pat_idx: p_idx,
                                    kind: DragKind::Move,
                                    orig_start: pat.start,
                                    orig_len: pat.len,
                                    mouse_start_x: mx,
                                });
                                cx.notify();
                            }
                        }),
                    )
                    .child(block);

                row = row.child(block_with_move);
            }

            // playhead in row
            row = row.child(
                div()
                    .absolute()
                    .left(px(ph_x))
                    .top_0()
                    .w(px(2.))
                    .h_full()
                    .bg(rgb(PLAYHEAD))
                    .opacity(0.7),
            );

            pattern_col = pattern_col.child(row);
        }

        let pattern_scroll = div()
            .id("pattern-scroll")
            .flex_1()
            .overflow_x_scroll()
            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, _w, cx| {
                if ev.pressed_button != Some(MouseButton::Left) {
                    return;
                }
                let x = f32::from(ev.position.x);
                if this.scrubbing {
                    this.scrub_to(x);
                    cx.notify();
                } else if this.drag.is_some() {
                    this.apply_drag(x);
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _w, cx| {
                    if this.drag.is_some() || this.scrubbing {
                        this.drag = None;
                        this.scrubbing = false;
                        cx.notify();
                    }
                }),
            )
            .child(pattern_col);

        // ── Beat sequencer panel ──────────────────────────────────────────────

        let beat_panel =
            if show_beat {
                let step_w = STEP_BTN + STEP_GAP;

                let mut panel = div()
                    .flex_none()
                    .h(px(BEAT_PANEL_H))
                    .flex()
                    .flex_col()
                    .bg(rgb(BG))
                    .border_t_1()
                    .border_color(rgb(BORDER));

                // Header row
                panel =
                    panel.child(
                        div()
                            .flex_none()
                            .h(px(28.))
                            .flex()
                            .flex_row()
                            .items_center()
                            .px_3()
                            .gap_4()
                            .bg(rgb(SURFACE))
                            .border_b_1()
                            .border_color(rgb(BORDER))
                            .child(
                                div()
                                    .w(px(HEADER_W - 12.))
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(ACCENT))
                                    .child("BEAT SEQUENCER"),
                            )
                            // step numbers
                            .child(div().flex().flex_row().items_center().children(
                                (0..STEPS).map(|s| {
                                    let extra = if s > 0 && s % 4 == 0 {
                                        STEP_GROUP_GAP
                                    } else {
                                        0.0
                                    };
                                    div()
                                        .w(px(step_w + extra))
                                        .pl(px(extra))
                                        .text_xs()
                                        .text_color(rgb(TEXT_DIM))
                                        .justify_center()
                                        .flex()
                                        .items_center()
                                        .child(if s % 4 == 0 {
                                            format!("{}", s / 4 + 1)
                                        } else {
                                            "·".into()
                                        })
                                }),
                            )),
                    );

                // Track rows
                let beat_scroll = div()
                    .id("beat-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .flex()
                    .flex_col();

                let mut beat_rows = beat_scroll;
                for t_idx in 0..n_tracks {
                    let track = &self.tracks[t_idx];
                    let color = track.color;
                    let muted = track.muted;
                    let steps = track.steps;
                    let name = track.name.clone();

                    // playback step highlight: which steps are active
                    let mut row = div()
                        .flex_none()
                        .h(px(24.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .border_b_1()
                        .border_color(rgb(BORDER))
                        // track label
                        .child(
                            div()
                                .flex_none()
                                .w(px(HEADER_W - 12.))
                                .px_2()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(if muted { TEXT_DIM } else { TEXT }))
                                .child(name),
                        );

                    // Step buttons
                    let mut steps_row = div().flex().flex_row().items_center();
                    for s in 0..STEPS {
                        let active = steps[s];
                        let is_playing_step = s == current_step;
                        let extra_left = if s > 0 && s % 4 == 0 {
                            STEP_GROUP_GAP
                        } else {
                            0.0
                        };

                        let btn_bg = match (active, is_playing_step) {
                            (true, true) => color | 0x404040, // bright
                            (true, false) => color,
                            (false, true) => SURFACE2 | 0x202020,
                            (false, false) => SURFACE2,
                        };
                        let btn_border = if is_playing_step {
                            PLAYHEAD
                        } else if active {
                            ACCENT_DIM
                        } else {
                            BORDER
                        };

                        steps_row = steps_row.child(
                            div()
                                .id(SharedString::from(format!("step-{t_idx}-{s}")))
                                .ml(px(extra_left + STEP_GAP))
                                .w(px(STEP_BTN))
                                .h(px(18.))
                                .bg(rgb(btn_bg))
                                .rounded_sm()
                                .border_1()
                                .border_color(rgb(btn_border))
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                                    this.tracks[t_idx].steps[s] ^= true;
                                    if this.tracks[t_idx].steps[s] {
                                        if let Some(audio) = &this.audio {
                                            audio.trigger(this.tracks[t_idx].sample);
                                        }
                                    }
                                    cx.notify();
                                })),
                        );
                    }

                    row = row.child(steps_row);
                    beat_rows = beat_rows.child(row);
                }

                panel.child(beat_rows)
            } else {
                div().flex_none().h(px(0.))
            };

        // ── Status bar ────────────────────────────────────────────────────────

        let status = div()
            .flex_none()
            .h(px(STATUS_H))
            .flex()
            .flex_row()
            .items_center()
            .px_3()
            .gap_4()
            .bg(rgb(SURFACE))
            .border_t_1()
            .border_color(rgb(BORDER))
            .child(dim(format!("{n_tracks} tracks")))
            .child(dim(format!("{total_bars} bars  ·  {bpm} BPM")))
            .child(div().flex_1())
            .child(dim("WAVE DAW  v0.1"));

        // ── Root ──────────────────────────────────────────────────────────────

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(BG))
            .child(transport)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    .child(headers)
                    .child(pattern_scroll),
            )
            .child(beat_panel)
            .child(status)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn dim(text: impl Into<SharedString>) -> impl IntoElement {
    div().text_xs().text_color(rgb(TEXT_DIM)).child(text.into())
}

fn transport_btn(
    icon: &'static str,
    id: &'static str,
    _active: bool,
    handler: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .id(SharedString::from(id))
        .px_2()
        .py_1()
        .rounded_sm()
        .bg(rgb(SURFACE2))
        .text_color(rgb(TEXT))
        .text_xs()
        .cursor_pointer()
        .hover(|s| s.bg(rgb(BORDER)))
        .on_click(handler)
        .child(icon)
}
