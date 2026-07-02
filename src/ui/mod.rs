mod beat;
pub mod core;
mod device;
mod headers;
mod pattern;
mod piano_roll;
mod transport;

use gpui::{
    AnyElement, ClickEvent, Context, IntoElement, MouseButton, MouseMoveEvent, MouseUpEvent,
    Render, SharedString, Window, div, prelude::*, px, rgb,
};

use crate::daw::Daw;
use crate::models::Instrument;
use crate::theme::*;
use core::button::tab_btn;
use core::text::dim;

impl Render for Daw {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let n_tracks = self.tracks.len();
        let total_bars = self.total_bars;
        let bpm = self.bpm;

        let status = div()
            .flex_none()
            .h(px(STATUS_H))
            .flex()
            .flex_row()
            .items_center()
            .px_3()
            .gap_4()
            .bg(rgb(SURFACE))
            .border_t_1()
            .border_color(rgb(BORDER))
            .child(dim(format!("{n_tracks} tracks")))
            .child(dim(format!("{total_bars} bars  ·  {bpm} BPM")))
            .child(div().flex_1())
            .child(dim(format!("◆ Enableaton v{}", env!("CARGO_PKG_VERSION"))));

        let transport = self.render_transport(cx);
        let headers = self.render_headers(cx);
        let pattern_scroll = self.render_pattern_area(cx);
        let bottom_panel: AnyElement = if !self.show_beat_editor {
            div().flex_none().into_any_element()
        } else {
            match self.selected_track.map(|i| &self.tracks[i].instrument) {
                Some(Instrument::DrumRack(_)) => self.render_beat_panel(cx).into_any_element(),
                Some(Instrument::Plugin(_)) if self.show_device => {
                    self.render_device_panel(cx).into_any_element()
                }
                Some(Instrument::Sampler(_) | Instrument::Plugin(_)) => {
                    self.render_piano_roll(cx).into_any_element()
                }
                None => div().flex_none().into_any_element(),
            }
        };

        div()
            .id("root")
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(BG))
            // Global mouse handlers for BPM / device-param drags (mouse can
            // move anywhere in the window while dragging).
            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, _w, cx| {
                if ev.pressed_button != Some(MouseButton::Left) {
                    return;
                }
                if let Some((start_y, start_bpm)) = this.bpm_drag_origin {
                    let delta = ((start_y - f32::from(ev.position.y)) / 1.5) as i32;
                    this.bpm = (start_bpm as i32 + delta).max(20).min(300) as u32;
                    cx.notify();
                }
                // Device panel parameter drag: full bar height = full param range.
                if let Some(drag) = this.param_drag.clone() {
                    if let Instrument::Plugin(p) = &this.tracks[drag.track_idx].instrument {
                        let spec = &p.params()[drag.param_idx];
                        let delta_y = drag.start_y - f32::from(ev.position.y);
                        let new_val = drag.start_val + delta_y / 120.0 * (spec.max - spec.min);
                        p.set_param(drag.param_idx, new_val);
                        cx.notify();
                    }
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _w, _cx| {
                    this.bpm_drag_origin = None;
                    this.param_drag = None;
                }),
            )
            .child(transport)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    .child(headers)
                    .child(pattern_scroll),
            )
            .child(bottom_panel)
            .child(status)
    }
}

// ── Shared UI helpers built on core components ───────────────────────────────

/// Pattern selector strip: one tab per pattern in the track's pool plus a
/// "＋" button that creates (and selects) a new pattern.
pub(super) fn pattern_tabs(
    t_idx: usize,
    names: &[String],
    active: usize,
    cx: &mut Context<Daw>,
) -> impl IntoElement {
    let mut row = div().flex().flex_row().items_center().gap_1();
    for (p_idx, name) in names.iter().enumerate() {
        row = row.child(tab_btn(
            SharedString::from(format!("pat-tab-{t_idx}-{p_idx}")),
            name.clone(),
            p_idx == active,
            cx.listener(move |this, _: &ClickEvent, _w, cx| {
                this.tracks[t_idx].active_pattern = p_idx;
                cx.notify();
            }),
        ));
    }
    row.child(tab_btn(
        SharedString::from(format!("pat-add-{t_idx}")),
        "＋",
        false,
        cx.listener(move |this, _: &ClickEvent, _w, cx| {
            let idx = this.tracks[t_idx].add_pattern();
            this.tracks[t_idx].active_pattern = idx;
            cx.notify();
        }),
    ))
}

/// NOTES / DEVICE panel switch for plugin tracks.
pub(super) fn view_tabs(device_active: bool, cx: &mut Context<Daw>) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(tab_btn(
            "view-tab-notes",
            "NOTES",
            !device_active,
            cx.listener(|this, _: &ClickEvent, _w, cx| {
                this.show_device = false;
                cx.notify();
            }),
        ))
        .child(tab_btn(
            "view-tab-device",
            "DEVICE",
            device_active,
            cx.listener(|this, _: &ClickEvent, _w, cx| {
                this.show_device = true;
                cx.notify();
            }),
        ))
}
