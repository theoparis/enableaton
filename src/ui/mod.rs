mod beat;
pub mod core;
mod headers;
mod pattern;
mod transport;

use gpui::{
    App, ClickEvent, Context, FontWeight, IntoElement, MouseButton, MouseMoveEvent, MouseUpEvent,
    Render, SharedString, Window, div, prelude::*, px, rgb,
};

use crate::daw::Daw;
use crate::theme::*;

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
        let beat_panel = self.render_beat_panel(cx);

        div()
            .id("root")
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(BG))
            // Global mouse handlers for BPM drag (mouse can move anywhere in window).
            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, _w, cx| {
                if ev.pressed_button != Some(MouseButton::Left) {
                    return;
                }
                if let Some((start_y, start_bpm)) = this.bpm_drag_origin {
                    let delta = ((start_y - f32::from(ev.position.y)) / 1.5) as i32;
                    this.bpm = (start_bpm as i32 + delta).max(20).min(300) as u32;
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _w, _cx| {
                    this.bpm_drag_origin = None;
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
            .child(beat_panel)
            .child(status)
    }
}

// ── Shared UI helpers ─────────────────────────────────────────────────────────

pub(super) fn dim(text: impl Into<SharedString>) -> impl IntoElement {
    div().text_xs().text_color(rgb(TEXT_DIM)).child(text.into())
}

pub(super) fn transport_btn(
    icon: &'static str,
    label: &'static str,
    id: &'static str,
    active: bool,
    handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(SharedString::from(id))
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .px_2()
        .py_1()
        .rounded_sm()
        .bg(rgb(if active { ACCENT_DIM } else { SURFACE2 }))
        .text_color(rgb(if active { ACCENT } else { TEXT }))
        .text_xs()
        .cursor_pointer()
        .hover(|s| s.bg(rgb(BORDER)))
        .on_click(handler)
        .child(div().text_sm().font_weight(FontWeight::BOLD).child(icon))
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(if active { ACCENT } else { TEXT_DIM }))
                .child(label),
        )
}

/// Small colored pill showing a 2-char sample-type abbreviation.
pub(super) fn sample_badge(label: &'static str, bg_color: u32) -> impl IntoElement {
    div()
        .flex_none()
        .px_1()
        .rounded_sm()
        .bg(rgb(bg_color))
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(0xc8dff0))
        .child(SharedString::from(label))
}
