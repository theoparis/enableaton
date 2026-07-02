use gpui::{ClickEvent, Context, FontWeight, IntoElement, SharedString, div, prelude::*, px, rgb};

use super::core::badge::sample_badge;
use super::core::button::icon_btn;
use super::core::panel::{chip, panel_header, panel_shell, panel_title};
use crate::daw::Daw;
use crate::models::Instrument;
use crate::theme::*;

fn default_pad_name(sample: crate::audio::SampleType) -> &'static str {
    use crate::audio::SampleType::*;
    match sample {
        Kick => "Kick",
        Snare => "Snare",
        HiHat => "Hi-Hat",
        OpenHat => "Open Hat",
        Clap => "Clap",
    }
}

struct PadRowData {
    t_idx: usize,
    pad_idx: usize,
    color: u32,
    muted: bool,
    name: String,
    badge_label: &'static str,
    badge_color: u32,
    steps: [bool; STEPS],
    playing_pattern: bool,
    current_step: usize,
}

impl Daw {
    pub(super) fn render_beat_panel(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let Some(t_idx) = self.selected_track else {
            return div().flex_none().h(px(BEAT_PANEL_H));
        };

        // Make sure the active pattern exists and carries a lane per pad.
        if self.tracks[t_idx].patterns.is_empty() {
            self.tracks[t_idx].add_pattern();
            self.tracks[t_idx].active_pattern = 0;
        }
        {
            let track = &mut self.tracks[t_idx];
            track.active_pattern = track.active_pattern.min(track.patterns.len() - 1);
            let n_lanes = track.instrument.n_lanes();
            let p_idx = track.active_pattern;
            let pat = &mut track.patterns[p_idx];
            if pat.lanes.len() < n_lanes {
                pat.lanes.resize(n_lanes, [false; STEPS]);
            }
        }

        let playing = self.playing;
        let play_pos = self.play_pos;
        let beats_per_bar = self.beats_per_bar;
        let step_w = STEP_BTN + STEP_GAP;
        let bar = (play_pos / beats_per_bar as f32).floor() as usize + 1;

        let track = &self.tracks[t_idx];
        let p_idx = track.active_pattern;
        let pat = &track.patterns[p_idx];
        let track_name = track.name.clone();
        let pattern_names: Vec<String> = track.patterns.iter().map(|p| p.name.clone()).collect();

        // Step highlight only while a clip of the edited pattern is playing.
        let (playing_pattern, current_step) = match track.clip_at(play_pos) {
            Some(c_idx) if playing && track.clips[c_idx].pattern == p_idx => {
                let elapsed = play_pos - track.clips[c_idx].start as f32;
                (true, (elapsed * 4.0).floor() as usize % pat.steps_in_loop())
            }
            _ => (false, usize::MAX),
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
                    steps: pat.lanes[pad_idx],
                    playing_pattern,
                    current_step,
                });
            }
        }

        // ── Header ──────────────────────────────────────────────────────────────
        let header = panel_header()
            .child(
                div()
                    .w(px(HEADER_W - 12.))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(panel_title(SharedString::from(format!(
                        "DRUM RACK: {}",
                        track_name.to_uppercase()
                    ))))
                    .child(chip("1/16"))
                    .child(chip(SharedString::from(format!("BAR {bar}")))),
            )
            .child(super::pattern_tabs(t_idx, &pattern_names, p_idx, cx))
            .child(div().flex_1())
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
            );

        let mut panel = panel_shell(BEAT_PANEL_H).child(header);

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
                playing_pattern,
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
                // Sample badge — click to cycle through the built-in sample bank.
                .child(
                    div()
                        .id(SharedString::from(format!("pad-sample-{t_idx}-{pad_idx}")))
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.7))
                        .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                            if let Instrument::DrumRack(pads) = &mut this.tracks[t_idx].instrument {
                                let next = pads[pad_idx].sample.next();
                                pads[pad_idx].sample = next;
                                pads[pad_idx].name = default_pad_name(next).into();
                                if let Some(audio) = &this.audio {
                                    audio.trigger(next);
                                }
                            }
                            cx.notify();
                        }))
                        .child(sample_badge(badge_label, badge_color)),
                )
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
                .child(icon_btn(
                    SharedString::from(format!("pad-mute-{t_idx}-{pad_idx}")),
                    "M",
                    muted,
                    cx.listener(move |this, _: &ClickEvent, _w, cx| {
                        if let Instrument::DrumRack(pads) = &mut this.tracks[t_idx].instrument {
                            pads[pad_idx].muted ^= true;
                        }
                        cx.notify();
                    }),
                ));

            let mut step_cells = div().flex().flex_row().items_center();
            for s in 0..STEPS {
                let active = steps[s];
                let is_playing_step = playing_pattern && s == current_step;
                let extra_left = if s > 0 && s % 4 == 0 {
                    STEP_GROUP_GAP
                } else {
                    0.0
                };

                let btn_bg = match (active, is_playing_step) {
                    (true, true) => color | 0x404040,
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
                            let track = &mut this.tracks[t_idx];
                            let p_idx = track.active_pattern;
                            let lane = &mut track.patterns[p_idx].lanes[pad_idx];
                            lane[s] ^= true;
                            if lane[s] {
                                if let Instrument::DrumRack(pads) = &track.instrument {
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

        panel = panel.child(beat_rows);
        panel
    }
}
