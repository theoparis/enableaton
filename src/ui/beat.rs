use gpui::{ClickEvent, Context, FontWeight, IntoElement, SharedString, div, prelude::*, px, rgb};

use super::sample_badge;
use crate::daw::Daw;
use crate::models::Instrument;
use crate::theme::*;

// Per-row data collected before building UI to avoid borrow conflicts.
struct StepRowData {
    t_idx: usize,
    /// `Some(i)` for a drum-rack pad row, `None` for a sampler track row.
    pad_idx: Option<usize>,
    color: u32,
    muted: bool,
    name: String,
    badge_label: &'static str,
    badge_color: u32,
    is_selected: bool,
    steps: [bool; STEPS],
    has_clip: bool,
    current_step: usize,
}

impl Daw {
    pub(super) fn render_beat_panel(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        if !self.show_beat_editor {
            return div().flex_none().h(px(0.));
        }

        let playing = self.playing;
        let play_pos = self.play_pos;
        let beats_per_bar = self.beats_per_bar;
        let step_w = STEP_BTN + STEP_GAP;
        let bar = (play_pos / beats_per_bar as f32).floor() as usize + 1;
        let selected = self.selected_track;

        // ── Collect rows ────────────────────────────────────────────────────────
        let mut rows: Vec<StepRowData> = Vec::new();
        for t_idx in 0..self.tracks.len() {
            let track = &self.tracks[t_idx];
            let active_pat_idx = Daw::active_pattern_idx(track, play_pos);
            let has_clip = active_pat_idx.is_some();
            let is_track_selected = selected == Some(t_idx);

            match &track.instrument {
                Instrument::Sampler(_) => {
                    let steps = active_pat_idx
                        .map(|i| track.patterns[i].steps)
                        .unwrap_or([false; STEPS]);
                    let current_step = if playing && has_clip {
                        let pat = &track.patterns[active_pat_idx.unwrap()];
                        let elapsed = play_pos - pat.start as f32;
                        (elapsed * 4.0).floor() as usize % STEPS
                    } else {
                        usize::MAX
                    };
                    rows.push(StepRowData {
                        t_idx,
                        pad_idx: None,
                        color: track.color,
                        muted: track.muted,
                        name: track.name.clone(),
                        badge_label: track.instrument.short_label(),
                        badge_color: track.instrument.badge_color(),
                        is_selected: is_track_selected,
                        steps,
                        has_clip,
                        current_step,
                    });
                }
                Instrument::DrumRack(pads) => {
                    let current_step = if playing && has_clip {
                        let pat = &track.patterns[active_pat_idx.unwrap()];
                        let elapsed = play_pos - pat.start as f32;
                        (elapsed * 4.0).floor() as usize % STEPS
                    } else {
                        usize::MAX
                    };
                    for (pad_idx, pad) in pads.iter().enumerate() {
                        rows.push(StepRowData {
                            t_idx,
                            pad_idx: Some(pad_idx),
                            color: pad.color,
                            muted: pad.muted || track.muted,
                            name: pad.name.clone(),
                            badge_label: pad.sample.short_label(),
                            badge_color: pad.sample.badge_color(),
                            is_selected: is_track_selected,
                            steps: pad.steps,
                            has_clip,
                            current_step,
                        });
                    }
                }
            }
        }

        // ── Header label ────────────────────────────────────────────────────────
        let editing_label = selected
            .map(|i| {
                let t = &self.tracks[i];
                SharedString::from(format!("EDITING: {}", t.name.to_uppercase()))
            })
            .unwrap_or_else(|| SharedString::from("BEAT SEQUENCER"));

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
                            let extra =
                                if s > 0 && s % 4 == 0 { STEP_GROUP_GAP } else { 0.0 };
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

        // ── Step rows ───────────────────────────────────────────────────────────
        let mut beat_rows = div()
            .id("beat-scroll")
            .flex_1()
            .overflow_y_scroll()
            .flex()
            .flex_col();

        for row in rows {
            let StepRowData {
                t_idx,
                pad_idx,
                color,
                muted,
                name,
                badge_label,
                badge_color,
                is_selected,
                steps,
                has_clip,
                current_step,
            } = row;

            let row_bg = if is_selected { 0x1a2a45u32 } else { BG };

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
                        .id(SharedString::from(format!(
                            "preview-{t_idx}-{}",
                            pad_idx.unwrap_or(usize::MAX)
                        )))
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
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                            this.selected_track = Some(t_idx);
                            if let Some(audio) = &this.audio {
                                match pad_idx {
                                    None => this.tracks[t_idx].instrument.trigger(audio),
                                    Some(pi) => {
                                        if let Instrument::DrumRack(pads) =
                                            &this.tracks[t_idx].instrument
                                        {
                                            audio.trigger(pads[pi].sample);
                                        }
                                    }
                                }
                            }
                            cx.notify();
                        }))
                        .child(SharedString::from(name)),
                )
                .child(
                    div()
                        .id(SharedString::from(format!(
                            "mute-{t_idx}-{}",
                            pad_idx.unwrap_or(usize::MAX)
                        )))
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
                            match pad_idx {
                                None => this.tracks[t_idx].muted ^= true,
                                Some(pi) => {
                                    if let Instrument::DrumRack(pads) =
                                        &mut this.tracks[t_idx].instrument
                                    {
                                        pads[pi].muted ^= true;
                                    }
                                }
                            }
                            cx.notify();
                        }))
                        .child("M"),
                );

            let mut step_cells = div().flex().flex_row().items_center();
            for s in 0..STEPS {
                let active = steps[s];
                let is_playing_step = s == current_step;
                let extra_left =
                    if s > 0 && s % 4 == 0 { STEP_GROUP_GAP } else { 0.0 };

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

                step_cells = step_cells.child(
                    div()
                        .id(SharedString::from(format!(
                            "step-{t_idx}-{}-{s}",
                            pad_idx.unwrap_or(usize::MAX)
                        )))
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
                            match pad_idx {
                                None => {
                                    if let Some(pi) =
                                        Daw::active_pattern_idx(&this.tracks[t_idx], pp)
                                    {
                                        this.tracks[t_idx].patterns[pi].steps[s] ^= true;
                                        if this.tracks[t_idx].patterns[pi].steps[s] {
                                            if let Some(audio) = &this.audio {
                                                this.tracks[t_idx].instrument.trigger(audio);
                                            }
                                        }
                                        cx.notify();
                                    }
                                }
                                Some(pi) => {
                                    if Daw::active_pattern_idx(&this.tracks[t_idx], pp)
                                        .is_some()
                                    {
                                        if let Instrument::DrumRack(pads) =
                                            &mut this.tracks[t_idx].instrument
                                        {
                                            pads[pi].steps[s] ^= true;
                                            if pads[pi].steps[s] {
                                                if let Some(audio) = &this.audio {
                                                    audio.trigger(pads[pi].sample);
                                                }
                                            }
                                        }
                                        cx.notify();
                                    }
                                }
                            }
                        })),
                );
            }

            beat_rows = beat_rows.child(
                div()
                    .flex_none()
                    .h(px(if is_selected { 28. } else { 24. }))
                    .flex()
                    .flex_row()
                    .items_center()
                    .border_b_1()
                    .border_color(rgb(if is_selected { ACCENT_DIM } else { BORDER }))
                    .bg(rgb(row_bg))
                    .child(header_cell)
                    .child(step_cells),
            );
        }

        panel.child(beat_rows)
    }
}
