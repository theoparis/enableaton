use gpui::{
    AnyElement, ClickEvent, Context, FontWeight, IntoElement, SharedString, div, prelude::*, px,
    rgb,
};

use super::core::button::{led, raised, vgrad};
use super::core::knob::knob;
use super::core::panel::{chip, fl_window, window_controls, window_title};
use crate::daw::Daw;
use crate::models::{Instrument, Note};
use crate::theme::*;

/// Width of the 16-step grid including group gaps.
pub(super) const STEPS_W: f32 = STEPS as f32 * (STEP_BTN + STEP_GAP) + 3.0 * STEP_GROUP_GAP;
const NAME_W: f32 = 96.0;

/// What a channel row edits.
enum RowKind {
    /// Step lane `lane` of the track's active pattern (drum pad or sampler).
    Steps { lane: usize, steps: [bool; STEPS] },
    /// Note preview of the track's active pattern (plugin instruments).
    Notes { notes: Vec<Note>, len: usize },
}

struct ChannelRow {
    t_idx: usize,
    /// Drum-rack pad index, if this row is a pad.
    pad: Option<usize>,
    name: String,
    color: u32,
    enabled: bool,
    kind: RowKind,
    current_step: Option<usize>,
    selected: bool,
}

impl Daw {
    /// Ensure track `t_idx` has an active pattern carrying one lane per pad.
    pub(super) fn ensure_active_pattern(&mut self, t_idx: usize) {
        let track = &mut self.tracks[t_idx];
        if track.patterns.is_empty() {
            track.add_pattern();
        }
        track.active_pattern = track.active_pattern.min(track.patterns.len() - 1);
        let n_lanes = track.instrument.n_lanes();
        let pat = &mut track.patterns[track.active_pattern];
        if pat.lanes.len() < n_lanes {
            pat.lanes.resize(n_lanes, [false; STEPS]);
        }
    }

    fn channel_rows(&mut self) -> Vec<ChannelRow> {
        let mut rows = Vec::new();
        for t_idx in 0..self.tracks.len() {
            self.ensure_active_pattern(t_idx);
            let track = &self.tracks[t_idx];
            let p_idx = track.active_pattern;
            let pat = &track.patterns[p_idx];
            let selected = self.selected_track == Some(t_idx);

            // Step highlight only while a clip of the edited pattern is playing.
            let current_step = match track.clip_at(self.play_pos) {
                Some(c) if self.playing && track.clips[c].pattern == p_idx => {
                    let elapsed = self.play_pos - track.clips[c].start as f32;
                    Some((elapsed * 4.0).floor() as usize % pat.steps_in_loop())
                }
                _ => None,
            };

            match &track.instrument {
                Instrument::DrumRack(pads) => {
                    for (pad_idx, pad) in pads.iter().enumerate() {
                        rows.push(ChannelRow {
                            t_idx,
                            pad: Some(pad_idx),
                            name: pad.name.clone(),
                            color: track.color,
                            enabled: !pad.muted && !track.muted,
                            kind: RowKind::Steps {
                                lane: pad_idx,
                                steps: pat.lanes[pad_idx],
                            },
                            current_step,
                            selected,
                        });
                    }
                }
                Instrument::Sampler(_) => rows.push(ChannelRow {
                    t_idx,
                    pad: None,
                    name: track.name.clone(),
                    color: track.color,
                    enabled: !track.muted,
                    kind: RowKind::Steps {
                        lane: 0,
                        steps: pat.lanes[0],
                    },
                    current_step,
                    selected,
                }),
                Instrument::Plugin(_) => rows.push(ChannelRow {
                    t_idx,
                    pad: None,
                    name: track.name.clone(),
                    color: track.color,
                    enabled: !track.muted,
                    kind: RowKind::Notes {
                        notes: pat.notes.clone(),
                        len: pat.len,
                    },
                    current_step,
                    selected,
                }),
            }
        }
        rows
    }

    pub(super) fn render_channel_rack(
        &mut self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let rows = self.channel_rows();

        let header = window_title("▦", "Channel rack")
            .child(chip("All"))
            .child(div().flex_1())
            .child(
                div()
                    .text_size(px(9.))
                    .text_color(rgb(TEXT_DIM))
                    .child("Swing"),
            )
            .child(knob(0.0, 14., ACCENT))
            .child(window_controls());

        let mut body = div()
            .id("rack-body")
            .flex_1()
            .flex()
            .flex_col()
            .py(px(3.))
            .gap(px(2.))
            .overflow_y_scroll();

        for (r_idx, row) in rows.into_iter().enumerate() {
            body = body.child(self.render_channel_row(r_idx, row, cx));
        }

        body = body.child(
            div()
                .id("add-track")
                .flex_none()
                .mt(px(4.))
                .mx(px(6.))
                .h(px(18.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(2.))
                .bg(raised(SURFACE2))
                .border_1()
                .border_color(rgb(BORDER))
                .text_size(px(12.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(TEXT))
                .cursor_pointer()
                .hover(|s| s.text_color(rgb(ACCENT)))
                .on_click(cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.add_track();
                    this.selected_track = Some(this.tracks.len() - 1);
                    cx.notify();
                }))
                .child("+"),
        );

        fl_window().child(header).child(body)
    }

    fn render_channel_row(
        &self,
        r_idx: usize,
        row: ChannelRow,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let ChannelRow {
            t_idx,
            pad,
            name,
            color,
            enabled,
            kind,
            current_step,
            selected,
        } = row;

        let mute = led(
            SharedString::from(format!("ch-led-{r_idx}")),
            enabled,
            10.,
            cx.listener(move |this, _: &ClickEvent, _w, cx| {
                match (pad, &mut this.tracks[t_idx].instrument) {
                    (Some(p), Instrument::DrumRack(pads)) => pads[p].muted ^= true,
                    _ => this.tracks[t_idx].muted ^= true,
                }
                cx.notify();
            }),
        );

        let name_btn = div()
            .id(SharedString::from(format!("ch-name-{r_idx}")))
            .flex_none()
            .w(px(NAME_W))
            .h(px(18.))
            .px(px(6.))
            .flex()
            .items_center()
            .rounded(px(2.))
            .bg(if selected {
                raised(0x6b7680)
            } else {
                raised(SURFACE2)
            })
            .border_1()
            .border_color(rgb(BORDER))
            .text_size(px(10.))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(rgb(if !enabled {
                TEXT_DIM
            } else if selected {
                TEXT_BRIGHT
            } else {
                TEXT
            }))
            .overflow_hidden()
            .cursor_pointer()
            .hover(|s| s.border_color(rgb(BEVEL)))
            .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                this.selected_track = Some(t_idx);
                if let Some(audio) = &this.audio {
                    match (pad, &this.tracks[t_idx].instrument) {
                        (Some(p), Instrument::DrumRack(pads)) => audio.trigger(pads[p].sample),
                        (_, inst) => inst.trigger(audio),
                    }
                }
                cx.notify();
            }))
            .child(SharedString::from(name));

        let content: AnyElement = match kind {
            RowKind::Steps { lane, steps } => {
                let mut cells = div().flex_none().flex().flex_row().items_center();
                for s in 0..STEPS {
                    let on = steps[s];
                    let is_now = current_step == Some(s);
                    let off_color = if (s / 4) % 2 == 0 {
                        STEP_OFF_A
                    } else {
                        STEP_OFF_B
                    };
                    let ml = if s == 0 {
                        0.0
                    } else if s % 4 == 0 {
                        STEP_GAP + STEP_GROUP_GAP
                    } else {
                        STEP_GAP
                    };
                    cells = cells.child(
                        div()
                            .id(SharedString::from(format!("step-{r_idx}-{s}")))
                            .flex_none()
                            .ml(px(ml))
                            .w(px(STEP_BTN))
                            .h(px(18.))
                            .rounded(px(2.))
                            .bg(if on {
                                vgrad(lighten(STEP_ON, 0.4), darken(STEP_ON, 0.12))
                            } else {
                                raised(off_color)
                            })
                            .border_1()
                            .border_color(rgb(if is_now { ACCENT } else { BORDER }))
                            .cursor_pointer()
                            .hover(|s| s.border_color(rgb(BEVEL)))
                            .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                                let track = &mut this.tracks[t_idx];
                                let p_idx = track.active_pattern;
                                let cell = &mut track.patterns[p_idx].lanes[lane][s];
                                *cell ^= true;
                                if *cell && let Some(audio) = &this.audio {
                                    match (pad, &track.instrument) {
                                        (Some(p), Instrument::DrumRack(pads)) => {
                                            audio.trigger(pads[p].sample)
                                        }
                                        (_, inst) => inst.trigger(audio),
                                    }
                                }
                                cx.notify();
                            })),
                    );
                }
                cells.into_any_element()
            }
            RowKind::Notes { notes, len } => note_preview(notes, len, color, STEPS_W, 18.)
                .bg(rgb(darken(color, 0.6)))
                .id(SharedString::from(format!("ch-notes-{r_idx}")))
                .rounded(px(2.))
                .border_1()
                .border_color(rgb(BORDER))
                .cursor_pointer()
                .hover(|s| s.border_color(rgb(BEVEL)))
                .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                    this.selected_track = Some(t_idx);
                    this.show_piano_roll = true;
                    this.show_device = false;
                    cx.notify();
                }))
                .into_any_element(),
        };

        div()
            .flex_none()
            .h(px(CHANNEL_ROW_H - 2.))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(4.))
            .px(px(6.))
            .child(mute)
            .child(knob(0.5, 15., TEXT_BRIGHT))
            .child(knob(0.78, 15., TEXT_BRIGHT))
            .child(
                div()
                    .flex_none()
                    .w(px(16.))
                    .h(px(14.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(2.))
                    .bg(rgb(LCD_BG))
                    .text_size(px(9.))
                    .text_color(rgb(LCD_TEXT))
                    .child(SharedString::from(format!("{}", t_idx + 1))),
            )
            .child(name_btn)
            .child(content)
            // Selection indicator (FL's green bar right of the row).
            .child(
                div()
                    .flex_none()
                    .w(px(5.))
                    .h(px(16.))
                    .rounded(px(1.))
                    .bg(rgb(if selected { LED_ON } else { LED_OFF }))
                    .border_1()
                    .border_color(rgb(BORDER)),
            )
            .child(div().flex_none().w(px(3.)).h(px(16.)).bg(rgb(color)))
    }
}

/// Miniature piano-roll rendering of `notes` across `len` beats, fitted to
/// `w`×`h`. Used by channel rows and playlist clips.
pub(super) fn note_preview(notes: Vec<Note>, len: usize, color: u32, w: f32, h: f32) -> gpui::Div {
    let lo = notes.iter().map(|n| n.pitch).min().unwrap_or(60);
    let hi = notes.iter().map(|n| n.pitch).max().unwrap_or(60);
    let span = (hi - lo).max(6) as f32;
    let beats = len.max(1) as f32;
    let note_h = (h / (span + 1.0)).clamp(2.0, 4.0);

    let mut area = div()
        .flex_none()
        .relative()
        .w(px(w))
        .h(px(h))
        .overflow_hidden();
    for b in 1..len {
        area = area.child(
            div()
                .absolute()
                .left(px(b as f32 / beats * w))
                .top_0()
                .w(px(1.))
                .h_full()
                .bg(rgb(darken(color, 0.45))),
        );
    }
    for n in notes.iter().filter(|n| n.start < beats) {
        let y = (1.0 - (n.pitch - lo) as f32 / span) * (h - note_h - 2.0) + 1.0;
        area = area.child(
            div()
                .absolute()
                .left(px(n.start / beats * w))
                .top(px(y))
                .w(px((n.duration / beats * w - 1.0).max(2.0)))
                .h(px(note_h))
                .rounded(px(1.))
                .bg(rgb(lighten(color, 0.6))),
        );
    }
    area
}
