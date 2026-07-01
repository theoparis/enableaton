use gpui::{ClickEvent, Context, FontWeight, IntoElement, SharedString, div, prelude::*, px, rgb};

use super::sample_badge;
use crate::daw::Daw;
use crate::models::Instrument;
use crate::theme::*;

struct PadRowData {
    t_idx: usize,
    pad_idx: usize,
    color: u32,
    muted: bool,
    name: String,
    badge_label: &'static str,
    badge_color: u32,
    steps: [bool; STEPS],
    has_clip: bool,
    current_step: usize,
}

impl Daw {
    pub(super) fn render_beat_panel(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let Some(t_idx) = self.selected_track else {
            return div().flex_none().h(px(BEAT_PANEL_H));
        };

        let playing = self.playing;
        let play_pos = self.play_pos;
        let beats_per_bar = self.beats_per_bar;
        let step_w = STEP_BTN + STEP_GAP;
        let bar = (play_pos / beats_per_bar as f32).floor() as usize + 1;

        let track = &self.tracks[t_idx];
        let track_name = track.name.clone();
        let active_pat_idx = Daw::active_pattern_idx(track, play_pos);
        let has_clip = active_pat_idx.is_some();
        let current_step = if playing && has_clip {
            let pat = &track.patterns[active_pat_idx.unwrap()];
            let elapsed = play_pos - pat.start as f32;
            (elapsed * 4.0).floor() as usize % STEPS
        } else {
            usize::MAX
        };

        // Collect pad rows from the drum rack.
        let mut rows: Vec<PadRowData> = Vec::new();
        if let Instrument::DrumRack(pads) = &track.instrument {
            for (pad_idx, pad) in pads.iter().enumerate() {
                rows.push(PadRowData {
                    t_idx,
                    pad_idx,
                    color: pad.color,
                    muted: pad.muted || track.muted,
                    name: pad.name.clone(),
                    badge_label: pad.sample.short_label(),
                    badge_color: pad.sample.badge_color(),
                    steps: pad.steps,
                    has_clip,
                    current_step,
                });
            }
        }

        // ── Header ──────────────────────────────────────────────────────────────
        let mut panel = div()
            .flex_none()
            .h(px(BEAT_PANEL_H))
            .flex()
            .flex_col()
            .bg(rgb(BG))
            .border_t_1()
            .border_color(rgb(BORDER));

        panel = panel.child(
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
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(ACCENT))
                                .child(SharedString::from(format!(
                                    "DRUM RACK: {}",
                                    track_name.to_uppercase()
                                ))),
                        )
                        .child(
                            div()
                                .px_1()
                                .rounded_sm()
                                .bg(rgb(SURFACE2))
                                .border_1()
                                .border_color(rgb(BORDER))
                                .text_xs()
                                .text_color(rgb(TEXT_DIM))
                                .child("1/16"),
                        )
                        .child(
                            div()
                                .px_1()
                                .rounded_sm()
                                .bg(rgb(SURFACE2))
                                .border_1()
                                .border_color(rgb(BORDER))
                                .text_xs()
                                .text_color(rgb(TEXT))
                                .child(SharedString::from(format!("BAR {bar}"))),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .children((0..STEPS).map(|s| {
                            let extra = if s > 0 && s % 4 == 0 {
                                STEP_GROUP_GAP
                            } else {
                                0.0
                            };
                            let is_beat = s % 4 == 0;
                            div()
                                .w(px(step_w + extra))
                                .pl(px(extra))
                                .text_xs()
                                .text_color(rgb(if is_beat { TEXT } else { TEXT_DIM }))
                                .font_weight(if is_beat {
                                    FontWeight::BOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .justify_center()
                                .flex()
                                .items_center()
                                .child(if is_beat {
                                    SharedString::from(format!("{}", s / 4 + 1))
                                } else {
                                    SharedString::from("·")
                                })
                        })),
                ),
        );

        // ── Pad rows ─────────────────────────────────────────────────────────────
        let mut beat_rows = div()
            .id("beat-scroll")
            .flex_1()
            .overflow_y_scroll()
            .flex()
            .flex_col();

        for row in rows {
            let PadRowData {
                t_idx,
                pad_idx,
                color,
                muted,
                name,
                badge_label,
                badge_color,
                steps,
                has_clip,
                current_step,
            } = row;

            let header_cell = div()
                .flex_none()
                .w(px(HEADER_W - 12.))
                .px_2()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(sample_badge(badge_label, badge_color))
                .child(
                    div()
                        .id(SharedString::from(format!("pad-preview-{t_idx}-{pad_idx}")))
                        .flex_1()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(if muted { TEXT_DIM } else { TEXT }))
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                            if let Instrument::DrumRack(pads) = &this.tracks[t_idx].instrument {
                                if let Some(audio) = &this.audio {
                                    audio.trigger(pads[pad_idx].sample);
                                }
                            }
                            cx.notify();
                        }))
                        .child(SharedString::from(name)),
                )
                .child(
                    div()
                        .id(SharedString::from(format!("pad-mute-{t_idx}-{pad_idx}")))
                        .flex_none()
                        .w(px(18.))
                        .h(px(18.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_sm()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(if muted { ACCENT } else { TEXT_DIM }))
                        .bg(rgb(if muted { ACCENT_DIM } else { SURFACE2 }))
                        .cursor_pointer()
                        .hover(|s| s.text_color(rgb(ACCENT)))
                        .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                            if let Instrument::DrumRack(pads) = &mut this.tracks[t_idx].instrument {
                                pads[pad_idx].muted ^= true;
                            }
                            cx.notify();
                        }))
                        .child("M"),
                );

            let mut step_cells = div().flex().flex_row().items_center();
            for s in 0..STEPS {
                let active = steps[s];
                let is_playing_step = s == current_step;
                let extra_left = if s > 0 && s % 4 == 0 {
                    STEP_GROUP_GAP
                } else {
                    0.0
                };

                let btn_bg = if !has_clip {
                    0x0d1520u32
                } else {
                    match (active, is_playing_step) {
                        (true, true) => color | 0x404040,
                        (true, false) => color,
                        (false, true) => SURFACE2 | 0x202020,
                        (false, false) => SURFACE2,
                    }
                };
                let btn_border = if is_playing_step && has_clip {
                    PLAYHEAD
                } else if active {
                    ACCENT_DIM
                } else {
                    BORDER
                };

                step_cells = step_cells.child(
                    div()
                        .id(SharedString::from(format!("step-{t_idx}-{pad_idx}-{s}")))
                        .ml(px(extra_left + STEP_GAP))
                        .w(px(STEP_BTN))
                        .h(px(20.))
                        .bg(rgb(btn_bg))
                        .rounded_sm()
                        .border_1()
                        .border_color(rgb(btn_border))
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                            if let Instrument::DrumRack(pads) = &mut this.tracks[t_idx].instrument {
                                pads[pad_idx].steps[s] ^= true;
                                if pads[pad_idx].steps[s] {
                                    if let Some(audio) = &this.audio {
                                        audio.trigger(pads[pad_idx].sample);
                                    }
                                }
                            }
                            cx.notify();
                        })),
                );
            }

            beat_rows = beat_rows.child(
                div()
                    .flex_none()
                    .h(px(36.))
                    .flex()
                    .flex_row()
                    .items_center()
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .bg(rgb(BG))
                    .child(header_cell)
                    .child(step_cells),
            );
        }

        panel.child(beat_rows)
    }
}
