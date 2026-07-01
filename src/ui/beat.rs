use gpui::{ClickEvent, Context, FontWeight, IntoElement, SharedString, div, prelude::*, px, rgb};

use super::sample_badge;
use crate::daw::Daw;
use crate::theme::*;

impl Daw {
    pub(super) fn render_beat_panel(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        if !self.show_beat_editor {
            return div().flex_none().h(px(0.));
        }

        let n_tracks = self.tracks.len();
        let playing = self.playing;
        let play_pos = self.play_pos;
        let beats_per_bar = self.beats_per_bar;
        let step_w = STEP_BTN + STEP_GAP;
        let bar = (play_pos / beats_per_bar as f32).floor() as usize + 1;
        let selected = self.selected_track;

        let mut panel = div()
            .flex_none()
            .h(px(BEAT_PANEL_H))
            .flex()
            .flex_col()
            .bg(rgb(BG))
            .border_t_1()
            .border_color(rgb(BORDER));

        // ── Header row ─────────────────────────────────────────────────────────
        let editing_label = selected
            .map(|i| SharedString::from(format!("EDITING: {}", self.tracks[i].name.to_uppercase())))
            .unwrap_or_else(|| SharedString::from("BEAT SEQUENCER"));

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
                                .text_color(rgb(if selected.is_some() { ACCENT } else { ACCENT }))
                                .child(editing_label),
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

        // ── Track rows ─────────────────────────────────────────────────────────
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
            let name = track.name.clone();
            let badge_label = track.instrument.short_label();
            let badge_color = track.instrument.badge_color();
            let is_selected = selected == Some(t_idx);

            let active_pat_idx = Daw::active_pattern_idx(track, play_pos);
            let steps = active_pat_idx
                .map(|i| track.patterns[i].steps)
                .unwrap_or([false; STEPS]);
            let has_clip = active_pat_idx.is_some();

            // Use pattern-relative step position so the highlight matches the actual sequencer.
            let current_step = if playing && has_clip {
                let pat = &track.patterns[active_pat_idx.unwrap()];
                let elapsed = play_pos - pat.start as f32;
                (elapsed * 4.0).floor() as usize % STEPS
            } else {
                usize::MAX
            };

            let row_bg = if is_selected { 0x1a2a45u32 } else { BG };

            let mut row = div()
                .flex_none()
                .h(px(if is_selected { 28. } else { 24. }))
                .flex()
                .flex_row()
                .items_center()
                .border_b_1()
                .border_color(rgb(if is_selected { ACCENT_DIM } else { BORDER }))
                .bg(rgb(row_bg))
                .child(
                    div()
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
                                .flex_1()
                                .text_xs()
                                .font_weight(if is_selected {
                                    FontWeight::BOLD
                                } else {
                                    FontWeight::SEMIBOLD
                                })
                                .text_color(rgb(if muted {
                                    TEXT_DIM
                                } else if is_selected {
                                    0xddf0ff
                                } else {
                                    TEXT
                                }))
                                .child(SharedString::from(name)),
                        ),
                );

            let mut steps_row = div().flex().flex_row().items_center();
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

                let btn_h = if is_selected { 22. } else { 18. };

                steps_row = steps_row.child(
                    div()
                        .id(SharedString::from(format!("step-{t_idx}-{s}")))
                        .ml(px(extra_left + STEP_GAP))
                        .w(px(STEP_BTN))
                        .h(px(btn_h))
                        .bg(rgb(btn_bg))
                        .rounded_sm()
                        .border_1()
                        .border_color(rgb(btn_border))
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                            this.selected_track = Some(t_idx);
                            let pp = this.play_pos;
                            if let Some(pat_idx) = Daw::active_pattern_idx(&this.tracks[t_idx], pp)
                            {
                                this.tracks[t_idx].patterns[pat_idx].steps[s] ^= true;
                                if this.tracks[t_idx].patterns[pat_idx].steps[s] {
                                    if let Some(audio) = &this.audio {
                                        this.tracks[t_idx].instrument.trigger(audio);
                                    }
                                }
                                cx.notify();
                            }
                        })),
                );
            }

            row = row.child(steps_row);
            beat_rows = beat_rows.child(row);
        }

        panel.child(beat_rows)
    }
}
