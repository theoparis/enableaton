use gpui::{
    ClickEvent, Context, FontWeight, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, ScrollWheelEvent, SharedString, canvas, div, point, prelude::*, px, rgb,
};

use super::core::button::{icon_btn, vgrad};
use super::core::panel::{chip, fl_window, window_controls, window_title};
use crate::daw::Daw;
use crate::models::{Instrument, Note, PianoRollResize};
use crate::theme::*;

const PIANO_KEY_W: f32 = 46.0;
const ROW_H: f32 = 14.0;
const VELOCITY_H: f32 = 44.0;
/// Fixed display velocity (notes don't carry velocity yet).
const DEFAULT_VELOCITY: f32 = 0.78;

// Pitch range rendered: C1 (24) to B5 (83) = 60 semitones.
const MIN_PITCH: u8 = 24;
const MAX_PITCH: u8 = 83;
const N_ROWS: usize = (MAX_PITCH - MIN_PITCH + 1) as usize;

const SNAP: f32 = 0.25;
const NOTE_DUR: f32 = 0.25;
const ZOOM_MIN: f32 = 0.5;
const ZOOM_MAX: f32 = 8.0;
const ZOOM_STEP: f32 = 0.25;

const WHITE_ROW: u32 = 0x3c4851;
const BLACK_ROW: u32 = 0x323d45;

fn is_black_key(pitch: u8) -> bool {
    matches!(pitch % 12, 1 | 3 | 6 | 8 | 10)
}

fn note_name(pitch: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    format!(
        "{}{}",
        NAMES[(pitch % 12) as usize],
        (pitch / 12) as i32 - 1
    )
}

impl Daw {
    pub(super) fn render_piano_roll(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let Some(t_idx) = self.selected_track else {
            return fl_window()
                .size_full()
                .child(
                    window_title("♪", "Piano roll")
                        .child(div().flex_1())
                        .child(window_controls()),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(11.))
                        .text_color(rgb(TEXT_DIM))
                        .child("Select a channel to edit its notes"),
                );
        };

        self.ensure_active_pattern(t_idx);
        let track = &self.tracks[t_idx];
        let p_idx = track.active_pattern;
        let pat = &track.patterns[p_idx];

        let track_name = track.name.clone();
        let color = track.color;
        let note_color = lighten(color, 0.3);
        let pat_len = pat.len;
        let pat_beats = pat_len as f32;
        let zoom = self.piano_roll_zoom;
        let beat_px = BEAT_PX * zoom;
        let total_w = pat_beats * beat_px;
        let notes_snapshot: Vec<Note> = pat.notes.clone();
        let is_plugin = matches!(track.instrument, Instrument::Plugin(_));
        let pattern_names: Vec<String> = track.patterns.iter().map(|p| p.name.clone()).collect();

        // Playhead in pattern-local coordinates: shown while any clip
        // instance of this pattern is under the timeline playhead.
        let local_ph = track.clips.iter().find_map(|c| {
            let cs = c.start as f32;
            (c.pattern == p_idx && self.play_pos >= cs && self.play_pos < cs + pat_beats)
                .then_some(self.play_pos - cs)
        });

        // Single scroll handle: .x = horizontal, .y = vertical (both ≤ 0).
        let scroll = self.piano_roll_scroll.clone();
        // Newly shown pattern: center the keyboard on its notes (or C4).
        if self.piano_roll_focus != Some((t_idx, p_idx)) {
            self.piano_roll_focus = Some((t_idx, p_idx));
            let center = match notes_snapshot.len() {
                0 => 60.0,
                n => notes_snapshot.iter().map(|n| n.pitch as f32).sum::<f32>() / n as f32,
            };
            let y = (MAX_PITCH as f32 - center) * ROW_H - 110.0;
            scroll.set_offset(point(px(0.), px(-y.max(0.))));
        }
        let scroll_x = f32::from(scroll.offset().x);
        let scroll_y = f32::from(scroll.offset().y);

        // ── Title bar ───────────────────────────────────────────────────────────
        let header = window_title(
            "♪",
            SharedString::from(format!("Piano roll - {track_name}")),
        )
        .child(super::pattern_tabs(t_idx, &pattern_names, p_idx, cx))
        .child(div().flex_1())
        .when(is_plugin, |d| d.child(super::view_tabs(false, cx)))
        .child(chip(SharedString::from(format!("{:.0}%", zoom * 100.0))))
        .child(icon_btn(
            "pr-zoom-out",
            "−",
            false,
            cx.listener(|this, _: &ClickEvent, _w, cx| {
                this.piano_roll_zoom = (this.piano_roll_zoom - ZOOM_STEP).max(ZOOM_MIN);
                cx.notify();
            }),
        ))
        .child(icon_btn(
            "pr-zoom-in",
            "+",
            false,
            cx.listener(|this, _: &ClickEvent, _w, cx| {
                this.piano_roll_zoom = (this.piano_roll_zoom + ZOOM_STEP).min(ZOOM_MAX);
                cx.notify();
            }),
        ))
        .child(window_controls());

        // ── Piano keyboard ──────────────────────────────────────────────────────
        let total_keys_h = N_ROWS as f32 * ROW_H;
        let mut keys_col = div()
            .flex_none()
            .w(px(PIANO_KEY_W))
            .h(px(total_keys_h))
            .flex()
            .flex_col();

        for row in 0..N_ROWS {
            let pitch = MAX_PITCH - row as u8;
            let black = is_black_key(pitch);
            let key = div()
                .flex_none()
                .h(px(ROW_H))
                .w_full()
                .relative()
                .bg(vgrad(0xf2f4f5, 0xd5dadd))
                .border_b_1()
                .border_color(rgb(if matches!(pitch % 12, 0 | 5) {
                    0x8a949b
                } else {
                    0xbfc6cb
                }))
                .when(pitch.is_multiple_of(12), |d| {
                    d.child(
                        div()
                            .absolute()
                            .right(px(3.))
                            .top(px(1.))
                            .text_size(px(8.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x3a4248))
                            .child(SharedString::from(note_name(pitch))),
                    )
                })
                .when(black, |d| {
                    d.child(
                        div()
                            .absolute()
                            .left_0()
                            .top_0()
                            .w(px(PIANO_KEY_W * 0.6))
                            .h_full()
                            .rounded_r(px(2.))
                            .bg(vgrad(0x3a4248, 0x15191c)),
                    )
                });
            keys_col = keys_col.child(key);
        }

        // ── Note grid (one pattern long) ─────────────────────────────────────────
        let mut grid = div()
            .relative()
            .flex_none()
            .w(px(total_w))
            .h(px(total_keys_h))
            .flex()
            .flex_col();

        let bpb = self.beats_per_bar;
        let origin = self.piano_roll_origin_x.clone();
        for row in 0..N_ROWS {
            let pitch = MAX_PITCH - row as u8;
            let black = is_black_key(pitch);

            let scroll_ref = scroll.clone();
            let origin_ref = origin.clone();
            let mut pitch_row = div()
                .id(SharedString::from(format!("pr-row-{pitch}")))
                .flex_none()
                .relative()
                .w(px(total_w))
                .h(px(ROW_H))
                .bg(rgb(if black { BLACK_ROW } else { WHITE_ROW }))
                .border_b_1()
                .border_color(rgb(if matches!(pitch % 12, 0 | 5) {
                    GRID_LINE
                } else {
                    mix(WHITE_ROW, GRID_LINE, 0.5)
                }))
                .cursor_crosshair()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, ev: &MouseDownEvent, _w, cx| {
                        let beat_px = BEAT_PX * this.piano_roll_zoom;
                        let sx = f32::from(scroll_ref.offset().x);
                        let raw_x = f32::from(ev.position.x) - origin_ref.get() - sx;
                        if raw_x < 0.0 {
                            return;
                        }

                        // Clicking near a note's right edge starts a resize.
                        const RESIZE_HIT: f32 = 6.0;
                        let resize_hit = this.tracks[t_idx].patterns[p_idx]
                            .notes
                            .iter()
                            .find(|n| {
                                n.pitch == pitch
                                    && (raw_x - (n.start + n.duration) * beat_px).abs() < RESIZE_HIT
                            })
                            .map(|n| (n.pitch, n.start, n.duration));

                        if let Some((note_pitch, note_start, orig_duration)) = resize_hit {
                            this.piano_roll_resize = Some(PianoRollResize {
                                track_idx: t_idx,
                                pattern_idx: p_idx,
                                note_pitch,
                                note_start,
                                orig_duration,
                                drag_start_x: f32::from(ev.position.x),
                            });
                            cx.notify();
                            return;
                        }

                        let beat = (raw_x / beat_px / SNAP).floor() * SNAP;
                        let notes = &mut this.tracks[t_idx].patterns[p_idx].notes;
                        if let Some(pos) = notes
                            .iter()
                            .position(|n| n.pitch == pitch && (n.start - beat).abs() < SNAP)
                        {
                            notes.remove(pos);
                        } else {
                            notes.push(Note {
                                pitch,
                                start: beat,
                                duration: NOTE_DUR,
                            });
                            if let Some(audio) = &this.audio {
                                this.tracks[t_idx].instrument.trigger_note(pitch, audio);
                            }
                        }
                        cx.notify();
                    }),
                );

            // Beat / bar grid lines.
            for beat in 1..pat_len {
                pitch_row = pitch_row.child(
                    div()
                        .absolute()
                        .left(px(beat as f32 * beat_px))
                        .top_0()
                        .w(px(1.))
                        .h_full()
                        .bg(rgb(if beat % bpb == 0 {
                            darken(GRID_LINE, 0.3)
                        } else {
                            GRID_LINE
                        })),
                );
            }

            for note in notes_snapshot.iter().filter(|n| n.pitch == pitch) {
                let nx = note.start * beat_px;
                let nw = (note.duration * beat_px - 1.0).max(3.0);
                pitch_row = pitch_row.child(
                    div()
                        .absolute()
                        .left(px(nx))
                        .top(px(1.0))
                        .w(px(nw))
                        .h(px(ROW_H - 2.0))
                        .flex()
                        .items_center()
                        .pl(px(3.))
                        .overflow_hidden()
                        .rounded(px(2.))
                        .bg(vgrad(lighten(note_color, 0.2), note_color))
                        .border_1()
                        .border_color(rgb(darken(note_color, 0.55)))
                        .text_size(px(8.))
                        .text_color(rgb(darken(note_color, 0.7)))
                        .when(nw > 22.0, |d| d.child(SharedString::from(note_name(pitch))))
                        // Resize handle: rightmost 6px, ew-resize cursor.
                        .child(
                            div()
                                .absolute()
                                .right(px(0.))
                                .top(px(0.))
                                .w(px(6.))
                                .h_full()
                                .cursor_col_resize(),
                        ),
                );
            }

            grid = grid.child(pitch_row);
        }

        if let Some(lp) = local_ph {
            grid = grid.child(
                div()
                    .absolute()
                    .left(px(lp * beat_px))
                    .top_0()
                    .w(px(1.))
                    .h_full()
                    .bg(rgb(PLAYHEAD)),
            );
        }

        // ── Layout ───────────────────────────────────────────────────────────────
        // Keyboard and velocity lane sit outside the scroll container and follow
        // its offset manually, so only one element owns the scroll handle.
        let keys_outer = div()
            .flex_none()
            .w(px(PIANO_KEY_W))
            .h_full()
            .overflow_hidden()
            .relative()
            .border_r_1()
            .border_color(rgb(BORDER))
            .child(keys_col.absolute().top(px(scroll_y)).left(px(0.)));

        let grid_area = div()
            .flex_1()
            .h_full()
            .relative()
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
                    .id("piano-roll-scroll")
                    .size_full()
                    .overflow_scroll()
                    .bg(rgb(INSET))
                    .track_scroll(&scroll)
                    .on_scroll_wheel(cx.listener(|this, ev: &ScrollWheelEvent, _w, cx| {
                        if ev.modifiers.control {
                            let delta = f32::from(-ev.delta.pixel_delta(px(1.0)).y);
                            let step = if delta > 0.0 { ZOOM_STEP } else { -ZOOM_STEP };
                            this.piano_roll_zoom =
                                (this.piano_roll_zoom + step).clamp(ZOOM_MIN, ZOOM_MAX);
                            cx.notify();
                        }
                    }))
                    .child(grid),
            );

        // Velocity lane: one stem per note at its start.
        let mut stems = div()
            .absolute()
            .top_0()
            .left(px(scroll_x))
            .w(px(total_w))
            .h_full();
        for beat in 0..pat_len {
            stems = stems.child(
                div()
                    .absolute()
                    .left(px(beat as f32 * beat_px))
                    .top_0()
                    .w(px(1.))
                    .h_full()
                    .bg(rgb(GRID_LINE)),
            );
        }
        for note in &notes_snapshot {
            let h = (VELOCITY_H - 8.0) * DEFAULT_VELOCITY;
            stems = stems
                .child(
                    div()
                        .absolute()
                        .left(px(note.start * beat_px + 3.))
                        .bottom_0()
                        .w(px(1.))
                        .h(px(h))
                        .bg(rgb(note_color)),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(note.start * beat_px))
                        .bottom(px(h - 3.))
                        .w(px(7.))
                        .h(px(7.))
                        .rounded_full()
                        .border_1()
                        .border_color(rgb(note_color)),
                );
        }
        let velocity = div()
            .flex_none()
            .h(px(VELOCITY_H))
            .flex()
            .flex_row()
            .border_t_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .flex_none()
                    .w(px(PIANO_KEY_W))
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgb(SURFACE))
                    .border_r_1()
                    .border_color(rgb(BORDER))
                    .text_size(px(8.))
                    .text_color(rgb(TEXT_DIM))
                    .child("VEL"),
            )
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .relative()
                    .overflow_hidden()
                    .bg(rgb(INSET))
                    .child(stems),
            );

        fl_window()
            .size_full()
            .on_mouse_move(cx.listener(move |this, ev: &MouseMoveEvent, _w, cx| {
                let Some(resize) = this.piano_roll_resize.clone() else {
                    return;
                };
                let beat_px = BEAT_PX * this.piano_roll_zoom;
                let dx = f32::from(ev.position.x) - resize.drag_start_x;
                let new_dur = ((resize.orig_duration + dx / beat_px) / SNAP)
                    .round()
                    .max(1.0)
                    * SNAP;
                // Locate note by (pitch, start) — immune to index shifts.
                if let Some(note) = this.tracks[resize.track_idx].patterns[resize.pattern_idx]
                    .notes
                    .iter_mut()
                    .find(|n| {
                        n.pitch == resize.note_pitch && (n.start - resize.note_start).abs() < 0.001
                    })
                {
                    note.duration = new_dur;
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _w, _cx| {
                    this.piano_roll_resize = None;
                }),
            )
            .child(header)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    .child(keys_outer)
                    .child(grid_area),
            )
            .child(velocity)
    }
}
