use gpui::{
    AnyElement, ClickEvent, Context, FontWeight, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, SharedString, canvas, div, prelude::*, px, rgb,
};

use super::channel_rack::note_preview;
use super::core::button::{led, raised, vgrad};
use super::core::panel::{chip, fl_window, window_title};
use crate::daw::Daw;
use crate::models::{DragKind, DragState, PatternData, Track};
use crate::theme::*;

const CLIP_BODY_H: f32 = TRACK_H - 4.0 - CLIP_TITLE_H;
const MUTED_GRAY: u32 = 0x5a6066;

/// Step-pattern preview for drum/sampler clips: one thin lane per pad,
/// repeated across the clip length like FL's pattern clips.
fn steps_preview(pat: &PatternData, color: u32, w: f32) -> gpui::Div {
    let n_lanes = pat.lanes.len().max(1);
    let lane_h = (CLIP_BODY_H - 4.0) / n_lanes as f32;
    let loop_steps = pat.steps_in_loop();
    let total_steps = pat.len * 4;
    let step_w = w / total_steps.max(1) as f32;
    let mut area = div()
        .flex_none()
        .relative()
        .w(px(w))
        .h(px(CLIP_BODY_H))
        .overflow_hidden();
    for (l, lane) in pat.lanes.iter().enumerate() {
        for s in 0..total_steps {
            if lane[s % loop_steps] {
                area = area.child(
                    div()
                        .absolute()
                        .left(px(s as f32 * step_w + 1.0))
                        .top(px(2.0 + l as f32 * lane_h))
                        .w(px((step_w - 2.0).max(1.5)))
                        .h(px((lane_h - 1.0).max(1.5)))
                        .rounded(px(1.))
                        .bg(rgb(lighten(color, 0.6))),
                );
            }
        }
    }
    area
}

fn clip_content(track: &Track, pat: &PatternData, color: u32, w: f32) -> AnyElement {
    if track.instrument.uses_piano_roll() && !pat.notes.is_empty() {
        note_preview(pat.notes.clone(), pat.len, color, w, CLIP_BODY_H - 2.0)
            .mt(px(1.))
            .into_any_element()
    } else {
        steps_preview(pat, color, w).into_any_element()
    }
}

impl Daw {
    pub(super) fn render_playlist(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let total_beats = self.total_bars * self.beats_per_bar;
        let total_w = BEAT_PX * total_beats as f32;
        let total_bars = self.total_bars;
        let bpb = self.beats_per_bar;
        let play_pos = self.play_pos;
        let n_tracks = self.tracks.len();
        let n_rows = n_tracks.max(MIN_PLAYLIST_ROWS);
        let ph_x = play_pos * BEAT_PX;
        let bar_w = bpb as f32 * BEAT_PX;

        let current = self.selected_track.map(|t| {
            let track = &self.tracks[t];
            let pat = track
                .patterns
                .get(track.active_pattern)
                .map_or("—", |p| p.name.as_str());
            format!("{} · {pat}", track.name)
        });

        // ── Title bar ──────────────────────────────────────────────────────────
        let header = window_title("▤", "Playlist - Arrangement")
            .when_some(current, |d, c| {
                d.child(
                    div()
                        .text_size(px(10.))
                        .text_color(rgb(TEXT_DIM))
                        .child(SharedString::from(format!("›  {c}"))),
                )
            })
            .child(div().flex_1())
            .child(
                div()
                    .text_size(px(9.))
                    .text_color(rgb(TEXT_DIM))
                    .child("Snap"),
            )
            .child(chip("Beat"))
            .child(super::close_ctl(
                "close-pl",
                |d| d.show_playlist = false,
                cx,
            ));

        // ── Track headers (left column) ────────────────────────────────────────
        let mut headers = div()
            .flex_none()
            .w(px(HEADER_W))
            .flex()
            .flex_col()
            .border_r_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .flex_none()
                    .h(px(RULER_H))
                    .flex()
                    .items_center()
                    .px(px(6.))
                    .bg(rgb(SURFACE))
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .text_size(px(9.))
                    .text_color(rgb(TEXT_DIM))
                    .child(SharedString::from(format!("{total_bars} bars"))),
            );

        for idx in 0..n_rows {
            let Some(t) = self.tracks.get(idx) else {
                // Empty placeholder track, FL-style.
                headers = headers.child(
                    div()
                        .flex_none()
                        .h(px(TRACK_H))
                        .px(px(6.))
                        .pt(px(3.))
                        .bg(raised(0x3d464d))
                        .border_b_1()
                        .border_color(rgb(BORDER))
                        .text_size(px(10.))
                        .text_color(rgb(TEXT_DIM))
                        .child(SharedString::from(format!("Track {}", idx + 1))),
                );
                continue;
            };
            let selected = self.selected_track == Some(idx);
            headers = headers.child(
                div()
                    .id(SharedString::from(format!("track-row-{idx}")))
                    .flex_none()
                    .h(px(TRACK_H))
                    .flex()
                    .flex_row()
                    .bg(if selected {
                        raised(0x56626b)
                    } else {
                        raised(0x454e56)
                    })
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                        this.selected_track = Some(idx);
                        this.preview_track(idx);
                        cx.notify();
                    }))
                    .child(div().flex_none().w(px(4.)).h_full().bg(rgb(t.color)))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .flex()
                            .flex_col()
                            .justify_between()
                            .px(px(5.))
                            .py(px(3.))
                            .child(
                                div()
                                    .text_size(px(10.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(if t.muted {
                                        TEXT_DIM
                                    } else if selected {
                                        TEXT_BRIGHT
                                    } else {
                                        TEXT
                                    }))
                                    .overflow_hidden()
                                    .child(SharedString::from(t.name.clone())),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_size(px(9.))
                                            .text_color(rgb(TEXT_DIM))
                                            .child(SharedString::from(t.instrument.short_label())),
                                    )
                                    .child(led(
                                        SharedString::from(format!("mute-{idx}")),
                                        !t.muted,
                                        9.,
                                        cx.listener(move |this, _: &ClickEvent, _w, cx| {
                                            cx.stop_propagation();
                                            this.tracks[idx].muted ^= true;
                                            cx.notify();
                                        }),
                                    )),
                            ),
                    ),
            );
        }

        // ── Ruler ──────────────────────────────────────────────────────────────
        let mut ruler = div()
            .id("ruler")
            .relative()
            .flex_none()
            .h(px(RULER_H))
            .w(px(total_w))
            .bg(vgrad(0x4a545c, 0x3e474e))
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
        for bar in 0..total_bars {
            let x = bar as f32 * bar_w;
            ruler = ruler
                .child(
                    div()
                        .absolute()
                        .left(px(x))
                        .bottom_0()
                        .w(px(1.))
                        .h(px(7.))
                        .bg(rgb(TEXT_DIM)),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(x + 3.))
                        .top(px(2.))
                        .text_size(px(10.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(TEXT))
                        .child(SharedString::from(format!("{}", bar + 1))),
                );
            for beat in 1..bpb {
                ruler = ruler.child(
                    div()
                        .absolute()
                        .left(px(x + beat as f32 * BEAT_PX))
                        .bottom_0()
                        .w(px(1.))
                        .h(px(3.))
                        .bg(rgb(TEXT_DIM)),
                );
            }
        }
        // Playhead marker (FL's downward triangle).
        ruler = ruler.child(
            div()
                .absolute()
                .left(px(ph_x - 5.))
                .bottom(px(-1.))
                .w(px(11.))
                .text_size(px(10.))
                .text_color(rgb(PLAYHEAD))
                .child("▼"),
        );

        // ── Grid background: 4-bar shading, bar and beat lines ─────────────────
        let grid_h = n_rows as f32 * TRACK_H;
        let mut grid_bg = div()
            .absolute()
            .left_0()
            .top(px(RULER_H))
            .w(px(total_w))
            .h(px(grid_h));
        for seg in 0..total_bars.div_ceil(4) {
            grid_bg = grid_bg.child(
                div()
                    .absolute()
                    .left(px(seg as f32 * 4.0 * bar_w))
                    .top_0()
                    .w(px(4.0 * bar_w))
                    .h_full()
                    .bg(rgb(if seg % 2 == 0 { GRID_A } else { GRID_B })),
            );
        }
        for beat in 0..total_beats {
            let is_bar = beat % bpb == 0;
            grid_bg = grid_bg.child(
                div()
                    .absolute()
                    .left(px(beat as f32 * BEAT_PX))
                    .top_0()
                    .w(px(1.))
                    .h_full()
                    .bg(rgb(if is_bar {
                        GRID_LINE
                    } else {
                        mix(GRID_A, GRID_LINE, 0.4)
                    })),
            );
        }
        for row in 1..=n_rows {
            grid_bg = grid_bg.child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(row as f32 * TRACK_H - 1.))
                    .w_full()
                    .h(px(1.))
                    .bg(rgb(GRID_LINE)),
            );
        }

        let mut pattern_col = div()
            .relative()
            .flex()
            .flex_col()
            .w(px(total_w))
            .child(ruler)
            .child(grid_bg);

        // ── Track rows with clips ──────────────────────────────────────────────
        for t_idx in 0..n_tracks {
            let track = &self.tracks[t_idx];
            let muted = track.muted;
            let color = if muted {
                mix(track.color, MUTED_GRAY, 0.75)
            } else {
                track.color
            };

            let mut row = div()
                .id(SharedString::from(format!("row-{t_idx}")))
                .relative()
                .flex_none()
                .h(px(TRACK_H))
                .w(px(total_w))
                // Click on empty row space inserts the track's active pattern.
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, ev: &MouseDownEvent, _w, cx| {
                        let beat = this.playlist_beat_at(f32::from(ev.position.x)).floor() as usize;
                        this.selected_track = Some(t_idx);
                        this.insert_clip(t_idx, beat);
                        cx.notify();
                    }),
                );

            let active_c_idx = track.clip_at(play_pos);
            for c_idx in 0..track.clips.len() {
                let clip = track.clips[c_idx];
                let pat = &track.patterns[clip.pattern];
                let x = clip.start as f32 * BEAT_PX;
                let w = (pat.len as f32 * BEAT_PX - 1.0).max(4.0);
                let is_active = self.playing && active_c_idx == Some(c_idx);
                let is_edited =
                    self.selected_track == Some(t_idx) && track.active_pattern == clip.pattern;
                let body_color = if is_active {
                    lighten(color, 0.1)
                } else {
                    color
                };

                let block = div()
                    .relative()
                    .size_full()
                    .flex()
                    .flex_col()
                    .rounded(px(2.))
                    .overflow_hidden()
                    .bg(rgb(darken(body_color, 0.3)))
                    .border_1()
                    .border_color(rgb(if is_edited {
                        lighten(body_color, 0.55)
                    } else {
                        darken(body_color, 0.5)
                    }))
                    .child(
                        div()
                            .flex_none()
                            .h(px(CLIP_TITLE_H))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(3.))
                            .px(px(3.))
                            .bg(vgrad(lighten(body_color, 0.1), body_color))
                            .overflow_hidden()
                            .text_size(px(9.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(TEXT_BRIGHT))
                            .child(div().text_size(px(7.)).child("▤"))
                            .child(SharedString::from(pat.name.clone())),
                    )
                    .child(clip_content(track, pat, body_color, w))
                    // Resize handle — resizes the pattern (all instances follow).
                    .child(
                        div()
                            .id(SharedString::from(format!("rsz-{t_idx}-{c_idx}")))
                            .absolute()
                            .right_0()
                            .top_0()
                            .w(px(6.))
                            .h_full()
                            .cursor_col_resize()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, ev: &MouseDownEvent, _w, cx| {
                                    cx.stop_propagation();
                                    let track = &this.tracks[t_idx];
                                    let clip = track.clips[c_idx];
                                    this.drag = Some(DragState {
                                        track_idx: t_idx,
                                        clip_idx: c_idx,
                                        kind: DragKind::Resize,
                                        orig_start: clip.start,
                                        orig_len: track.patterns[clip.pattern].len,
                                        mouse_start_x: f32::from(ev.position.x),
                                    });
                                    cx.notify();
                                }),
                            ),
                    );

                row = row.child(
                    div()
                        .id(SharedString::from(format!("clip-{t_idx}-{c_idx}")))
                        .absolute()
                        .left(px(x))
                        .top(px(2.))
                        .w(px(w))
                        .h(px(TRACK_H - 4.))
                        .cursor_grab()
                        // Left: select pattern for editing + start move drag.
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, ev: &MouseDownEvent, _w, cx| {
                                cx.stop_propagation();
                                let track = &mut this.tracks[t_idx];
                                let clip = track.clips[c_idx];
                                this.selected_track = Some(t_idx);
                                track.active_pattern = clip.pattern;
                                this.drag = Some(DragState {
                                    track_idx: t_idx,
                                    clip_idx: c_idx,
                                    kind: DragKind::Move,
                                    orig_start: clip.start,
                                    orig_len: track.patterns[clip.pattern].len,
                                    mouse_start_x: f32::from(ev.position.x),
                                });
                                cx.notify();
                            }),
                        )
                        // Right: remove this clip instance (pattern stays in the pool).
                        .on_mouse_down(
                            MouseButton::Right,
                            cx.listener(move |this, _: &MouseDownEvent, _w, cx| {
                                cx.stop_propagation();
                                this.tracks[t_idx].clips.remove(c_idx);
                                cx.notify();
                            }),
                        )
                        .child(block),
                );
            }

            pattern_col = pattern_col.child(row);
        }

        // Playhead line across all rows.
        pattern_col = pattern_col.child(
            div()
                .absolute()
                .left(px(ph_x))
                .top(px(RULER_H))
                .w(px(1.))
                .h(px(grid_h))
                .bg(rgb(PLAYHEAD)),
        );

        let scroll_handle = self.pattern_scroll.clone();
        let origin = self.playlist_origin_x.clone();
        let grid = div()
            .flex_1()
            .relative()
            .overflow_hidden()
            // Record the grid's window-space left edge for mouse → beat mapping.
            .child(
                canvas(
                    move |bounds, _, _| origin.set(f32::from(bounds.left())),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .child(
                div()
                    .id("pattern-scroll")
                    .size_full()
                    .overflow_x_scroll()
                    .track_scroll(&scroll_handle)
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
                    .child(pattern_col),
            );

        fl_window().flex_1().h_full().child(header).child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .overflow_hidden()
                .child(headers)
                .child(grid),
        )
    }
}
