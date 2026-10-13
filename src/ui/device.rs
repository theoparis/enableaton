use gpui::{
    ClickEvent, Context, FontWeight, IntoElement, MouseButton, MouseDownEvent, SharedString, div,
    prelude::*, px, rgb,
};

use super::core::badge::sample_badge;
use super::core::button::{led, raised, tab_btn};
use super::core::knob::knob;
use super::core::panel::{fl_window, window_title};
use crate::audio::{ParamKind, SampleType};
use crate::daw::Daw;
use crate::models::{Instrument, ParamDrag};
use crate::theme::*;

const CELL_W: f32 = 54.0;
const KNOB: f32 = 30.0;

fn default_pad_name(sample: SampleType) -> &'static str {
    match sample {
        SampleType::Kick => "Kick",
        SampleType::Snare => "Snare",
        SampleType::HiHat => "Hi-Hat",
        SampleType::OpenHat => "Open Hat",
        SampleType::Clap => "Clap",
    }
}

/// Grouped box inside the channel settings window.
fn section(title: &'static str) -> gpui::Div {
    div()
        .flex_none()
        .flex()
        .flex_col()
        .gap(px(6.))
        .p(px(6.))
        .rounded(px(3.))
        .bg(rgb(INSET))
        .border_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .text_size(px(9.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(ACCENT))
                .child(title),
        )
}

impl Daw {
    /// Plugin parameters ("Channel settings") for the selected synth track.
    pub(super) fn render_device_panel(
        &mut self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let Some(t_idx) = self.selected_track else {
            return fl_window().size_full();
        };
        let track = &self.tracks[t_idx];
        let Instrument::Plugin(plugin) = &track.instrument else {
            return fl_window().size_full();
        };

        let specs = plugin.params();
        let values: Vec<f32> = (0..specs.len()).map(|i| plugin.get_param(i)).collect();

        let header = window_title(
            "⚙",
            SharedString::from(format!(
                "Channel settings - {} ({})",
                track.name,
                plugin.short_label()
            )),
        )
        .child(div().flex_1())
        .child(super::view_tabs(true, cx))
        .child(super::close_ctl(
            "close-dev",
            |d| d.show_piano_roll = false,
            cx,
        ));

        // Group params by section, preserving first-seen order.
        let mut groups: Vec<(&'static str, Vec<usize>)> = Vec::new();
        for (i, spec) in specs.iter().enumerate() {
            match groups.iter_mut().find(|(g, _)| *g == spec.group) {
                Some((_, idxs)) => idxs.push(i),
                None => groups.push((spec.group, vec![i])),
            }
        }

        let mut body = div()
            .id("device-body")
            .flex_1()
            .flex()
            .flex_row()
            .flex_wrap()
            .content_start()
            .gap(px(6.))
            .p(px(6.))
            .overflow_y_scroll();

        for (group_name, idxs) in groups {
            let mut cells = div().flex().flex_row().gap(px(4.));
            for p_idx in idxs {
                let spec = &specs[p_idx];
                let val = values[p_idx];
                let cell = match spec.kind {
                    ParamKind::Float(unit) => {
                        let norm = ((val - spec.min) / (spec.max - spec.min)).clamp(0.0, 1.0);
                        div()
                            .flex_none()
                            .w(px(CELL_W))
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(3.))
                            .child(
                                knob(norm, KNOB, ACCENT)
                                    .id(SharedString::from(format!("param-{t_idx}-{p_idx}")))
                                    .cursor_ns_resize()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, ev: &MouseDownEvent, _w, cx| {
                                            this.param_drag = Some(ParamDrag {
                                                track_idx: t_idx,
                                                param_idx: p_idx,
                                                start_y: f32::from(ev.position.y),
                                                start_val: val,
                                            });
                                            cx.notify();
                                        }),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(9.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(TEXT))
                                    .child(spec.name),
                            )
                            .child(
                                div()
                                    .text_size(px(9.))
                                    .text_color(rgb(LCD_TEXT))
                                    .child(SharedString::from(format!("{val:.2}{unit}"))),
                            )
                            .into_any_element()
                    }
                    ParamKind::Choice(labels) => {
                        let sel = (val.round() as usize).min(labels.len() - 1);
                        let mut col = div().flex_none().flex().flex_col().gap(px(2.)).child(
                            div()
                                .text_size(px(9.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(TEXT))
                                .child(spec.name),
                        );
                        for (c_idx, label) in labels.iter().enumerate() {
                            col = col.child(tab_btn(
                                SharedString::from(format!("choice-{t_idx}-{p_idx}-{c_idx}")),
                                *label,
                                c_idx == sel,
                                cx.listener(move |this, _: &ClickEvent, _w, cx| {
                                    if let Instrument::Plugin(p) = &this.tracks[t_idx].instrument {
                                        p.set_param(p_idx, c_idx as f32);
                                        // Preview the new sound.
                                        if let Some(audio) = &this.audio {
                                            audio.trigger_plugin(
                                                p.as_ref(),
                                                Some(60),
                                                None,
                                                this.gain(t_idx),
                                            );
                                        }
                                    }
                                    cx.notify();
                                }),
                            ));
                        }
                        col.into_any_element()
                    }
                };
                cells = cells.child(cell);
            }
            body = body.child(section(group_name).child(cells));
        }

        fl_window().size_full().child(header).child(body)
    }

    /// Pad/sample assignment for the selected drum rack track.
    pub(super) fn render_drum_settings(
        &mut self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let Some(t_idx) = self.selected_track else {
            return fl_window().size_full();
        };
        let track = &self.tracks[t_idx];
        let Instrument::DrumRack(pads) = &track.instrument else {
            return fl_window().size_full();
        };

        let header = window_title(
            "⚙",
            SharedString::from(format!("Channel settings - {} (Drum rack)", track.name)),
        )
        .child(div().flex_1())
        .child(super::close_ctl(
            "close-dev",
            |d| d.show_piano_roll = false,
            cx,
        ));

        let mut list = div()
            .id("drum-pads")
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(3.))
            .p(px(6.))
            .overflow_y_scroll();

        for (pad_idx, pad) in pads.iter().enumerate() {
            let mut samples = div().flex().flex_row().gap(px(3.));
            for &sample in SampleType::all() {
                samples = samples.child(tab_btn(
                    SharedString::from(format!("pad-sample-{t_idx}-{pad_idx}-{sample:?}")),
                    default_pad_name(sample),
                    pad.sample == sample,
                    cx.listener(move |this, _: &ClickEvent, _w, cx| {
                        if let Instrument::DrumRack(pads) = &mut this.tracks[t_idx].instrument {
                            pads[pad_idx].sample = sample;
                            pads[pad_idx].name = default_pad_name(sample).into();
                        }
                        if let Some(audio) = &this.audio {
                            audio.trigger(sample, this.gain(t_idx));
                        }
                        cx.notify();
                    }),
                ));
            }

            list = list.child(
                div()
                    .flex_none()
                    .h(px(26.))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.))
                    .px(px(6.))
                    .rounded(px(2.))
                    .bg(raised(SURFACE2))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .child(led(
                        SharedString::from(format!("pad-mute-{t_idx}-{pad_idx}")),
                        !pad.muted,
                        10.,
                        cx.listener(move |this, _: &ClickEvent, _w, cx| {
                            if let Instrument::DrumRack(pads) = &mut this.tracks[t_idx].instrument {
                                pads[pad_idx].muted ^= true;
                            }
                            cx.notify();
                        }),
                    ))
                    .child(sample_badge(
                        pad.sample.short_label(),
                        darken(track.color, 0.2),
                    ))
                    .child(
                        div()
                            .w(px(64.))
                            .text_size(px(10.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(if pad.muted { TEXT_DIM } else { TEXT }))
                            .child(SharedString::from(pad.name.clone())),
                    )
                    .child(samples),
            );
        }

        fl_window().size_full().child(header).child(list)
    }
}
