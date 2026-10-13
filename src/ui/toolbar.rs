use gpui::{
    ClickEvent, Context, FontWeight, IntoElement, MouseButton, MouseDownEvent, SharedString, div,
    prelude::*, px, rgb,
};

use super::core::button::{tool_btn, vgrad};
use super::core::panel::lcd;
use super::core::text::dim;
use crate::daw::Daw;
use crate::theme::*;

const MENU_ITEMS: &[&str] = &[
    "FILE", "EDIT", "ADD", "PATTERNS", "VIEW", "OPTIONS", "TOOLS", "HELP",
];
const SCOPE_BARS: usize = 36;

fn separator() -> impl IntoElement {
    div()
        .flex_none()
        .w(px(1.))
        .h(px(30.))
        .mx(px(4.))
        .bg(rgb(BORDER))
}

impl Daw {
    pub(super) fn render_toolbar(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let bpm = self.bpm;
        let playing = self.playing;
        let recording = self.recording;
        let song_mode = self.song_mode;
        let bpb = self.beats_per_bar as f32;
        let pos = self.play_pos;
        let bar = (pos / bpb).floor() as usize + 1;
        let step = ((pos % bpb) * 4.0).floor() as usize + 1;
        let tick = ((pos * 4.0).fract() * 24.0).floor() as usize;

        let pattern_label = match self.selected_track {
            Some(t) => {
                let track = &self.tracks[t];
                let pat = track
                    .patterns
                    .get(track.active_pattern)
                    .map_or("—", |p| p.name.as_str());
                format!("{} · {pat}", track.name)
            }
            None => "(none)".to_string(),
        };

        // ── Left: menu + hint line + master volume ─────────────────────────────
        let menu = div()
            .flex_none()
            .w(px(300.))
            .flex()
            .flex_col()
            .justify_center()
            .gap(px(3.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(7.))
                    .children(MENU_ITEMS.iter().map(|m| {
                        div()
                            .text_size(px(10.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(TEXT))
                            .cursor_pointer()
                            .hover(|s| s.text_color(rgb(ACCENT)))
                            .child(*m)
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.))
                    .child(
                        div()
                            .flex_1()
                            .px(px(4.))
                            .rounded(px(2.))
                            .bg(rgb(INSET))
                            .text_size(px(10.))
                            .text_color(rgb(TEXT_DIM))
                            .overflow_hidden()
                            .child(SharedString::from(format!(
                                "untitled.enb — {} channels",
                                self.tracks.len()
                            ))),
                    )
                    .child(dim("VOL"))
                    .child(div().w(px(56.)).child(self.master_slider.clone())),
            );

        // ── Transport ──────────────────────────────────────────────────────────
        let mode_switch = div().flex_none().flex().flex_col().gap(px(2.)).children(
            [("PAT", !song_mode), ("SONG", song_mode)].map(|(label, on)| {
                div()
                    .id(SharedString::from(format!("mode-{label}")))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(3.))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                        this.song_mode = label == "SONG";
                        cx.notify();
                    }))
                    .child(
                        div()
                            .w(px(7.))
                            .h(px(7.))
                            .rounded_full()
                            .bg(rgb(if on { ACCENT } else { LED_OFF }))
                            .border_1()
                            .border_color(rgb(BORDER)),
                    )
                    .child(
                        div()
                            .text_size(px(9.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(if on { TEXT_BRIGHT } else { TEXT_DIM }))
                            .child(label),
                    )
            }),
        );

        let transport = div()
            .flex_none()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(3.))
            .child(tool_btn(
                "btn-play",
                if playing { "❚❚" } else { "▶" },
                26.,
                playing,
                LED_ON,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.toggle_play(cx);
                    cx.notify();
                }),
            ))
            .child(tool_btn(
                "btn-stop",
                "■",
                26.,
                false,
                ACCENT,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.playing = false;
                    this.play_pos = 0.0;
                    this.stop_tick();
                    cx.notify();
                }),
            ))
            .child(tool_btn(
                "btn-rec",
                "●",
                26.,
                recording,
                RED,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.recording = !this.recording;
                    cx.notify();
                }),
            ));

        // Tempo — drag up/down to change.
        let tempo = lcd()
            .id("bpm-drag")
            .h(px(26.))
            .flex_col()
            .justify_center()
            .items_end()
            .cursor_ns_resize()
            .hover(|s| s.border_color(rgb(ACCENT_DIM)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, ev: &MouseDownEvent, _w, _cx| {
                    this.bpm_drag_origin = Some((f32::from(ev.position.y), this.bpm));
                }),
            )
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::BOLD)
                    .child(SharedString::from(format!("{bpm}.000"))),
            )
            .child(
                div()
                    .text_size(px(7.))
                    .text_color(rgb(TEXT_DIM))
                    .mt(px(-3.))
                    .child("TEMPO"),
            );

        let time = lcd()
            .h(px(32.))
            .gap(px(4.))
            .child(
                div()
                    .text_size(px(20.))
                    .font_weight(FontWeight::BOLD)
                    .child(SharedString::from(format!("{bar}:{step:02}"))),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(7.))
                            .text_color(rgb(TEXT_DIM))
                            .child("B:S:T"),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(FontWeight::BOLD)
                            .child(SharedString::from(format!("{tick:02}"))),
                    ),
            );

        // Decorative oscilloscope that wiggles while playing.
        let scope = div()
            .flex_none()
            .w(px(96.))
            .h(px(30.))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(1.))
            .px(px(3.))
            .rounded(px(2.))
            .bg(rgb(LCD_BG))
            .border_1()
            .border_color(rgb(BORDER))
            .children((0..SCOPE_BARS).map(|i| {
                let amp = if playing {
                    let t = pos * 9.0 + i as f32 * 0.55;
                    (t.sin() * (t * 2.3).cos()).abs() * (1.0 - (pos * 4.0).fract() * 0.6)
                } else {
                    0.04
                };
                div()
                    .flex_1()
                    .h(px((amp * 24.0).max(1.0)))
                    .bg(rgb(0xb7c9d6))
            }));

        let pattern_sel = lcd()
            .h(px(22.))
            .w(px(150.))
            .gap(px(4.))
            .child(div().text_size(px(9.)).text_color(rgb(ACCENT)).child("▤"))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .text_size(px(10.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(SharedString::from(pattern_label)),
            )
            .child(div().text_size(px(8.)).text_color(rgb(TEXT_DIM)).child("▼"));

        let windows = div()
            .flex_none()
            .flex()
            .flex_row()
            .gap(px(2.))
            .child(tool_btn(
                "win-browser",
                "☰",
                22.,
                self.show_browser,
                ACCENT,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.show_browser ^= true;
                    cx.notify();
                }),
            ))
            .child(tool_btn(
                "win-rack",
                "▦",
                22.,
                self.show_channel_rack,
                ACCENT,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.show_channel_rack ^= true;
                    cx.notify();
                }),
            ))
            .child(tool_btn(
                "win-piano",
                "♪",
                22.,
                self.show_piano_roll,
                ACCENT,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.show_piano_roll ^= true;
                    cx.notify();
                }),
            ))
            .child(tool_btn(
                "win-mixer",
                "▥",
                22.,
                self.show_mixer,
                ACCENT,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.show_mixer ^= true;
                    cx.notify();
                }),
            ));

        div()
            .flex_none()
            .h(px(TOOLBAR_H))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .px(px(8.))
            .bg(vgrad(0x4b555d, 0x3c454c))
            .border_b_1()
            .border_color(rgb(BORDER))
            .child(menu)
            .child(separator())
            .child(mode_switch)
            .child(transport)
            .child(tempo)
            .child(separator())
            .child(time)
            .child(scope)
            .child(separator())
            .child(pattern_sel)
            .child(windows)
            .child(div().flex_1())
            .child(
                div()
                    .flex_none()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_size(px(14.))
                            .text_color(rgb(ACCENT))
                            .font_weight(FontWeight::BOLD)
                            .child("◆"),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(TEXT))
                            .child(SharedString::from(format!(
                                "Enableaton {}",
                                env!("CARGO_PKG_VERSION")
                            ))),
                    ),
            )
    }
}
