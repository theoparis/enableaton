use gpui::{
    ClickEvent, Context, FontWeight, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, ScrollWheelEvent, SharedString, div, prelude::*, px, rgb,
};

use super::core::button::icon_btn;
use super::core::panel::{chip, panel_header, panel_shell, panel_title};
use crate::daw::Daw;
use crate::models::{Instrument, Note, PianoRollResize};
use crate::theme::*;

const PIANO_KEY_W: f32 = 48.0;
const PANEL_H: f32 = 320.0;
const HEADER_H: f32 = 28.0;
const NOTE_AREA_H: f32 = PANEL_H - HEADER_H;
const ROW_H: f32 = 16.0;

// Pitch range rendered: C1 (24) to B5 (83) = 60 semitones.
const MIN_PITCH: u8 = 24;
const MAX_PITCH: u8 = 83;
const N_ROWS: usize = (MAX_PITCH - MIN_PITCH + 1) as usize;

const SNAP: f32 = 0.25;
const NOTE_DUR: f32 = 0.25;
const ZOOM_MIN: f32 = 0.5;
const ZOOM_MAX: f32 = 8.0;
const ZOOM_STEP: f32 = 0.25;

fn is_black_key(pitch: u8) -> bool {
    matches!(pitch % 12, 1 | 3 | 6 | 8 | 10)
}

fn note_label(pitch: u8) -> Option<String> {
    if pitch % 12 == 0 {
        let octave = (pitch / 12) as i32 - 1;
        Some(format!("C{octave}"))
    } else {
        None
    }
}

impl Daw {
    pub(super) fn render_piano_roll(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let Some(t_idx) = self.selected_track else {
            return div().flex_none().h(px(PANEL_H));
        };

        if self.tracks[t_idx].patterns.is_empty() {
            self.tracks[t_idx].add_pattern();
            self.tracks[t_idx].active_pattern = 0;
        }
        let track = &self.tracks[t_idx];
        let p_idx = track.active_pattern.min(track.patterns.len() - 1);
        let pat = &track.patterns[p_idx];

        let track_name = track.name.clone();
        let color = track.color;
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

        // Single scroll handle: .x = horizontal, .y = vertical.
        let scroll = self.piano_roll_scroll.clone();
        let scroll_y = f32::from(scroll.offset().y); // ≤ 0 when scrolled down

        // ── Header ──────────────────────────────────────────────────────────────
        let header = panel_header()
            .child(panel_title(SharedString::from(format!(
                "PIANO ROLL: {}",
                track_name.to_uppercase()
            ))))
            .child(super::pattern_tabs(t_idx, &pattern_names, p_idx, cx))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(TEXT_DIM))
                    .child("click to add · click note to remove"),
            )
            .child(div().flex_1())
            .when(is_plugin, |d| d.child(super::view_tabs(false, cx)))
            .child(chip(SharedString::from(format!("{pat_len}b"))))
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
            ));

        // ── Piano keys column ────────────────────────────────────────────────────
        let total_keys_h = N_ROWS as f32 * ROW_H;
        let mut keys_col = div()
            .flex_none()
            .w(px(PIANO_KEY_W))
            .h(px(total_keys_h))
            .flex()
            .flex_col()
            .bg(rgb(SURFACE))
            .border_r_1()
            .border_color(rgb(BORDER));

        for row in 0..N_ROWS {
            let pitch = MAX_PITCH - row as u8;
            let black = is_black_key(pitch);
            let label = note_label(pitch);

            keys_col = keys_col.child(
                div()
                    .flex_none()
                    .h(px(ROW_H))
                    .w_full()
                    .flex()
                    .items_center()
                    .px_2()
                    .bg(rgb(if black { 0x0e1620 } else { SURFACE }))
                    .border_b_1()
                    .border_color(rgb(if pitch % 12 == 0 { BORDER } else { 0x1a2535 }))
                    .when_some(label, |d, lbl| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(rgb(ACCENT))
                                .font_weight(FontWeight::BOLD)
                                .child(SharedString::from(lbl)),
                        )
                    }),
            );
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
        for row in 0..N_ROWS {
            let pitch = MAX_PITCH - row as u8;
            let black = is_black_key(pitch);
            let is_c = pitch % 12 == 0;

            let row_notes: Vec<Note> = notes_snapshot
                .iter()
                .filter(|n| n.pitch == pitch)
                .cloned()
                .collect();

            let scroll_ref = scroll.clone();
            let mut pitch_row = div()
                .id(SharedString::from(format!("pr-row-{pitch}")))
                .flex_none()
                .relative()
                .w(px(total_w))
                .h(px(ROW_H))
                .bg(rgb(if black { 0x111820 } else { BG }))
                .border_b_1()
                .border_color(rgb(if is_c { BORDER } else { 0x192030 }))
                .cursor_crosshair()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, ev: &MouseDownEvent, _w, cx| {
                        let zoom = this.piano_roll_zoom;
                        let beat_px = BEAT_PX * zoom;
                        let sx = f32::from(scroll_ref.offset().x);
                        // content_x = screen_x - PIANO_KEY_W - scroll_x (scroll_x ≤ 0)
                        let raw_x = f32::from(ev.position.x) - PIANO_KEY_W - sx;
                        if raw_x < 0.0 {
                            return;
                        }

                        // Check if clicking near the right edge of any note on this row.
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

            // Bar grid lines.
            for bar in 0..=(pat_len / bpb) {
                let x = bar as f32 * bpb as f32 * beat_px;
                pitch_row = pitch_row.child(
                    div()
                        .absolute()
                        .left(px(x))
                        .top_0()
                        .w_px()
                        .h_full()
                        .bg(rgb(BORDER))
                        .opacity(0.35),
                );
            }

            // Notes.
            for note in &row_notes {
                let nx = note.start * beat_px;
                let nw = (note.duration * beat_px - 1.0).max(3.0);
                pitch_row = pitch_row.child(
                    div()
                        .absolute()
                        .left(px(nx))
                        .top(px(1.0))
                        .w(px(nw))
                        .h(px(ROW_H - 2.0))
                        .bg(rgb(color))
                        .rounded_sm()
                        .border_l_2()
                        .border_color(rgb(ACCENT))
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

        // Playhead (pattern-local).
        if let Some(lp) = local_ph {
            grid = grid.child(
                div()
                    .absolute()
                    .left(px(lp * beat_px))
                    .top_0()
                    .w(px(2.))
                    .h_full()
                    .bg(rgb(PLAYHEAD))
                    .opacity(0.8),
            );
        }

        // ── Layout ───────────────────────────────────────────────────────────────
        // Keys column is outside the scrollable area; synced vertically by reading
        // scroll.offset().y and applying it as an absolute top offset. This avoids
        // the dual-scroller conflict where two track_scroll elements fight over the
        // shared handle's x component.
        let keys_outer = div()
            .flex_none()
            .w(px(PIANO_KEY_W))
            .h(px(NOTE_AREA_H))
            .overflow_hidden()
            .relative()
            .child(
                // scroll_y ≤ 0; top(scroll_y) shifts content up when scrolled down.
                keys_col.absolute().top(px(scroll_y)).left(px(0.)),
            );

        // Grid: single overflow_scroll container (both X and Y) with one handle.
        let grid_area = div()
            .id("piano-roll-scroll")
            .flex_1()
            .h(px(NOTE_AREA_H))
            .overflow_scroll()
            .track_scroll(&scroll)
            .on_scroll_wheel(cx.listener(|this, ev: &ScrollWheelEvent, _w, cx| {
                if ev.modifiers.control {
                    let delta = f32::from(-ev.delta.pixel_delta(px(1.0)).y);
                    let step = if delta > 0.0 { ZOOM_STEP } else { -ZOOM_STEP };
                    this.piano_roll_zoom = (this.piano_roll_zoom + step).clamp(ZOOM_MIN, ZOOM_MAX);
                    cx.notify();
                }
            }))
            .child(grid);

        let note_area = div()
            .flex_1()
            .flex()
            .flex_row()
            .overflow_hidden()
            .child(keys_outer)
            .child(grid_area);

        panel_shell(PANEL_H)
            .on_mouse_move(cx.listener(move |this, ev: &MouseMoveEvent, _w, cx| {
                let Some(ref resize) = this.piano_roll_resize.clone() else {
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
            .child(note_area)
    }
}
