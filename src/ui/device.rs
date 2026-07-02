use gpui::{
    ClickEvent, Context, FontWeight, IntoElement, MouseButton, MouseDownEvent, SharedString, div,
    prelude::*, px, rgb,
};

use super::core::panel::{panel_header, panel_shell, panel_title};
use crate::audio::ParamKind;
use crate::daw::Daw;
use crate::models::{Instrument, ParamDrag};
use crate::theme::*;

pub(super) const DEVICE_PANEL_H: f32 = 320.0;
const CELL_W: f32 = 64.0;
const BAR_H: f32 = 120.0;

impl Daw {
    pub(super) fn render_device_panel(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let Some(t_idx) = self.selected_track else {
            return div().flex_none().h(px(DEVICE_PANEL_H));
        };
        let track = &self.tracks[t_idx];
        let track_name = track.name.clone();
        let Instrument::Plugin(plugin) = &track.instrument else {
            return div().flex_none().h(px(DEVICE_PANEL_H));
        };

        let specs = plugin.params();
        let values: Vec<f32> = (0..specs.len()).map(|i| plugin.get_param(i)).collect();
        let plugin_label = plugin.short_label().to_string();

        // ── Header with NOTES / DEVICE tabs ─────────────────────────────────────
        let header = panel_header()
            .child(panel_title(SharedString::from(format!(
                "DEVICE: {} · {}",
                track_name.to_uppercase(),
                plugin_label
            ))))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(TEXT_DIM))
                    .child("drag values vertically"),
            )
            .child(div().flex_1())
            .child(super::view_tabs(true, cx));

        // ── Parameter groups ─────────────────────────────────────────────────────
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
            .items_start()
            .gap_4()
            .p_3()
            .overflow_x_scroll();

        for (group_name, idxs) in groups {
            let mut cells = div().flex().flex_row().gap_2();

            for p_idx in idxs {
                let spec = &specs[p_idx];
                let val = values[p_idx];

                let cell: gpui::AnyElement = match spec.kind {
                    ParamKind::Float(unit) => {
                        let norm = ((val - spec.min) / (spec.max - spec.min)).clamp(0.0, 1.0);
                        div()
                            .id(SharedString::from(format!("param-{t_idx}-{p_idx}")))
                            .flex_none()
                            .w(px(CELL_W))
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(TEXT_DIM))
                                    .child(spec.name),
                            )
                            .child(
                                // Vertical value bar; drag up/down to change.
                                div()
                                    .w(px(18.))
                                    .h(px(BAR_H))
                                    .rounded_sm()
                                    .bg(rgb(SURFACE2))
                                    .border_1()
                                    .border_color(rgb(BORDER))
                                    .cursor_ns_resize()
                                    .hover(|s| s.border_color(rgb(ACCENT_DIM)))
                                    .flex()
                                    .flex_col()
                                    .justify_end()
                                    .overflow_hidden()
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
                                    )
                                    .child(
                                        div()
                                            .w_full()
                                            .h(px(BAR_H * norm))
                                            .bg(rgb(ACCENT_DIM)),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(TEXT))
                                    .child(SharedString::from(format!("{val:.2}{unit}"))),
                            )
                            .into_any_element()
                    }
                    ParamKind::Choice(labels) => {
                        let sel = (val.round() as usize).min(labels.len() - 1);
                        let mut col = div()
                            .flex_none()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(TEXT_DIM))
                                    .child(spec.name),
                            );
                        for (c_idx, label) in labels.iter().enumerate() {
                            let active = c_idx == sel;
                            col = col.child(
                                div()
                                    .id(SharedString::from(format!(
                                        "choice-{t_idx}-{p_idx}-{c_idx}"
                                    )))
                                    .px_2()
                                    .py_px()
                                    .w(px(CELL_W - 8.))
                                    .rounded_sm()
                                    .text_xs()
                                    .text_color(rgb(if active { ACCENT } else { TEXT_DIM }))
                                    .bg(rgb(if active { ACCENT_DIM } else { SURFACE2 }))
                                    .border_1()
                                    .border_color(rgb(if active { ACCENT } else { BORDER }))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(BORDER)))
                                    .flex()
                                    .justify_center()
                                    .on_click(cx.listener(
                                        move |this, _: &ClickEvent, _w, cx| {
                                            if let Instrument::Plugin(p) =
                                                &this.tracks[t_idx].instrument
                                            {
                                                p.set_param(p_idx, c_idx as f32);
                                                // Preview the new sound.
                                                if let Some(audio) = &this.audio {
                                                    audio.trigger_plugin(p.as_ref(), Some(60));
                                                }
                                            }
                                            cx.notify();
                                        },
                                    ))
                                    .child(SharedString::from(*label)),
                            );
                        }
                        col.into_any_element()
                    }
                };
                cells = cells.child(cell);
            }

            body = body.child(
                div()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_2()
                    .rounded_sm()
                    .bg(rgb(SURFACE))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            .child(group_name),
                    )
                    .child(cells),
            );
        }

        panel_shell(DEVICE_PANEL_H).child(header).child(body)
    }
}
