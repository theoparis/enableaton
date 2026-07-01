use gpui::{
    ClickEvent, Context, FontWeight, IntoElement, MouseButton, MouseDownEvent, div, prelude::*, px,
    rgb,
};

use super::{dim, transport_btn};
use crate::daw::Daw;
use crate::theme::*;

impl Daw {
    pub(super) fn render_transport(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let bpm = self.bpm;
        let playing = self.playing;
        let recording = self.recording;
        let bar = (self.play_pos / self.beats_per_bar as f32).floor() as usize + 1;
        let beat = (self.play_pos % self.beats_per_bar as f32).floor() as usize + 1;
        let show_beat = self.show_beat_editor;

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px_3()
            .h(px(TRANSPORT_H))
            .flex_none()
            .bg(rgb(SURFACE))
            .border_b_1()
            .border_color(rgb(BORDER))
            // App logo
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .mr_2()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(ACCENT))
                            .font_weight(FontWeight::BOLD)
                            .child("◆"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(TEXT))
                            .child("Enableaton"),
                    ),
            )
            // Rewind
            .child(transport_btn(
                "◀◀",
                "",
                "rew",
                false,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.play_pos = 0.0;
                    cx.notify();
                }),
            ))
            // Play / Pause
            .child(
                div()
                    .id("btn-play")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .px_3()
                    .py_1()
                    .rounded_sm()
                    .bg(rgb(if playing { ACCENT } else { SURFACE2 }))
                    .text_color(rgb(if playing { 0x000000 } else { TEXT }))
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(ACCENT_DIM)))
                    .on_click(cx.listener(|this, _: &ClickEvent, _w, cx| {
                        this.toggle_play(cx);
                        cx.notify();
                    }))
                    .child(if playing { "⏸" } else { "▶" })
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(if playing { "PAUSE" } else { "PLAY" }),
                    ),
            )
            // Stop
            .child(transport_btn(
                "■",
                "STOP",
                "stop",
                false,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.playing = false;
                    this.play_pos = 0.0;
                    this.stop_tick();
                    cx.notify();
                }),
            ))
            // Record
            .child(
                div()
                    .id("btn-rec")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .bg(rgb(if recording { RED_MUTED } else { SURFACE2 }))
                    .text_color(rgb(0xdd4444))
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(RED_MUTED)))
                    .on_click(cx.listener(|this, _: &ClickEvent, _w, cx| {
                        this.recording = !this.recording;
                        cx.notify();
                    }))
                    .child("●")
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(if recording { 0xdd4444 } else { TEXT_DIM }))
                            .child("REC"),
                    ),
            )
            .child(div().w_px().h_6().bg(rgb(BORDER)).mx_2())
            // BPM — drag up/down to change (Ableton-style)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(dim("BPM"))
                    .child(
                        div()
                            .id("bpm-drag")
                            .px_2()
                            .py_px()
                            .bg(rgb(BG))
                            .rounded_sm()
                            .border_1()
                            .border_color(rgb(BORDER))
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            // Vertical resize cursor signals drag-to-change
                            .cursor_ns_resize()
                            .hover(|s| s.border_color(rgb(ACCENT_DIM)))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, ev: &MouseDownEvent, _w, _cx| {
                                    this.bpm_drag_origin =
                                        Some((f32::from(ev.position.y), this.bpm));
                                }),
                            )
                            .child(format!("{bpm}")),
                    ),
            )
            // Position display
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .ml_2()
                    .child(dim("POS"))
                    .child(
                        div()
                            .px_2()
                            .py_px()
                            .bg(rgb(BG))
                            .rounded_sm()
                            .border_1()
                            .border_color(rgb(BORDER))
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(TEXT))
                            .child(format!("{bar} : {beat} : 00")),
                    ),
            )
            .child(div().flex_1())
            // Beat editor toggle
            .child(
                div()
                    .id("beat-toggle")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .bg(rgb(if show_beat { ACCENT_DIM } else { SURFACE2 }))
                    .text_color(rgb(if show_beat { ACCENT } else { TEXT_DIM }))
                    .text_xs()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(BORDER)))
                    .on_click(cx.listener(|this, _: &ClickEvent, _w, cx| {
                        this.show_beat_editor = !this.show_beat_editor;
                        cx.notify();
                    }))
                    .child("⊞")
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("BEAT"),
                    ),
            )
            .child(div().w_px().h_6().bg(rgb(BORDER)).mx_2())
            // Master volume
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(dim("MASTER"))
                    .child(div().w_24().child(self.master_slider.clone())),
            )
    }
}
