use gpui::{
    ClickEvent, Context, FontWeight, IntoElement, MouseButton, MouseDownEvent, SharedString, div,
    linear_color_stop, linear_gradient, prelude::*, px, rgb,
};

use super::core::button::{led, raised, vgrad};
use super::core::knob::knob;
use super::core::panel::{chip, fl_window, lcd, window_title};
use crate::daw::Daw;
use crate::models::UiDrag;
use crate::theme::*;

const SLOTS: usize = 8;
const SLOT_PANEL_W: f32 = 196.0;
const MASTER_STRIP_W: f32 = 52.0;
/// Fader position shown on empty inserts.
const DEFAULT_FADER: f32 = 0.8;

struct StripData {
    /// Insert number; 0 = master.
    num: usize,
    name: String,
    color: Option<u32>,
    enabled: bool,
    selected: bool,
    level: (f32, f32),
    fader: f32,
    pan: f32,
    /// Track backing this insert, if any.
    track: Option<usize>,
}

/// Stereo peak meter: two thin bars filled bottom-up green → yellow → red.
fn meter(level: (f32, f32)) -> impl IntoElement {
    let bar = |v: f32| {
        div()
            .flex_none()
            .w(px(4.))
            .h_full()
            .flex()
            .flex_col()
            .justify_end()
            .bg(rgb(LCD_BG))
            .child(
                div()
                    .w_full()
                    .h(gpui::relative(v.clamp(0.0, 1.0)))
                    .bg(linear_gradient(
                        0.,
                        linear_color_stop(rgb(0x5fcf4a), 0.),
                        linear_color_stop(rgb(0xd8e35a), 0.85),
                    )),
            )
    };
    div()
        .flex_none()
        .h_full()
        .flex()
        .flex_row()
        .gap(px(1.))
        .p(px(1.))
        .rounded(px(1.))
        .bg(rgb(BORDER))
        .child(bar(level.0))
        .child(bar(level.1))
}

/// Vertical fader groove with a cap at `value` (0..1).
fn fader(value: f32) -> impl IntoElement {
    div()
        .flex_none()
        .relative()
        .w(px(14.))
        .h_full()
        .child(
            div()
                .absolute()
                .left(px(5.))
                .top_0()
                .w(px(4.))
                .h_full()
                .rounded_full()
                .bg(rgb(LCD_BG)),
        )
        .child(
            div()
                .absolute()
                .left_0()
                .top(gpui::relative(1.0 - value.clamp(0.0, 1.0)))
                .mt(px(-5.))
                .w(px(14.))
                .h(px(10.))
                .rounded(px(2.))
                .bg(vgrad(0xb4bdc3, 0x6b757c))
                .border_1()
                .border_color(rgb(BORDER))
                .child(div().mt(px(3.)).h(px(1.)).w_full().bg(rgb(0x3a4248))),
        )
}

impl Daw {
    /// Fake but transport-reactive meter levels for a track insert.
    fn track_level(&self, t_idx: usize) -> (f32, f32) {
        let t = &self.tracks[t_idx];
        if !self.playing || t.muted || self.pattern_at(t_idx, self.play_pos).is_none() {
            return (0.0, 0.0);
        }
        let decay = 1.0 - (self.play_pos * 2.0).fract();
        let base = 0.45 + 0.4 * decay;
        let wobble = ((self.play_pos * 7.0 + t_idx as f32).sin() * 0.06).abs();
        (base, (base - wobble).max(0.0))
    }

    pub(super) fn render_mixer(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let n_tracks = self.tracks.len();
        let n_inserts = n_tracks.max(MIN_MIXER_INSERTS);

        let mut master_level = (0.0f32, 0.0f32);
        let mut strips = Vec::with_capacity(n_inserts + 1);
        for num in 1..=n_inserts {
            let t_idx = num - 1;
            let Some(track) = self.tracks.get(t_idx) else {
                strips.push(StripData {
                    num,
                    name: format!("Insert {num}"),
                    color: None,
                    enabled: true,
                    selected: false,
                    level: (0.0, 0.0),
                    fader: DEFAULT_FADER,
                    pan: 0.0,
                    track: None,
                });
                continue;
            };
            let level = self.track_level(t_idx);
            master_level.0 = master_level.0.max(level.0);
            master_level.1 = master_level.1.max(level.1);
            strips.push(StripData {
                num,
                name: track.name.clone(),
                color: Some(track.color),
                enabled: !track.muted,
                selected: self.selected_track == Some(t_idx),
                level,
                fader: self.mix[t_idx].volume,
                pan: self.mix[t_idx].pan,
                track: Some(t_idx),
            });
        }
        strips.insert(
            0,
            StripData {
                num: 0,
                name: "Master".into(),
                color: None,
                enabled: true,
                selected: self.selected_track.is_none(),
                level: (
                    master_level.0 * self.master_vol,
                    master_level.1 * self.master_vol,
                ),
                fader: self.master_vol,
                pan: 0.0,
                track: None,
            },
        );

        let selected_name = self
            .selected_track
            .map_or("Master".to_string(), |t| self.tracks[t].name.clone());

        let mut strip_row = div()
            .id("mixer-strips")
            .flex_1()
            .flex()
            .flex_row()
            .gap(px(1.))
            .p(px(3.))
            .overflow_x_scroll()
            .bg(rgb(INSET));
        for strip in strips {
            strip_row = strip_row.child(self.render_strip(strip, cx));
        }

        // ── Insert effect slots (right panel) ──────────────────────────────────
        let mut slots = div()
            .flex_none()
            .w(px(SLOT_PANEL_W))
            .flex()
            .flex_col()
            .gap(px(2.))
            .p(px(4.))
            .border_l_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .text_size(px(10.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(TEXT_BRIGHT))
                    .child(SharedString::from(format!("Mixer - {selected_name}"))),
            )
            .child(lcd().h(px(16.)).text_size(px(9.)).child("(none)"));
        for slot in 1..=SLOTS {
            slots = slots.child(
                div()
                    .flex_none()
                    .h(px(15.))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(4.))
                    .px(px(4.))
                    .rounded(px(2.))
                    .bg(raised(SURFACE2))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .child(div().text_size(px(7.)).text_color(rgb(TEXT_DIM)).child("▼"))
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(9.))
                            .text_color(rgb(TEXT_DIM))
                            .child(SharedString::from(format!("Slot {slot}"))),
                    )
                    .child(knob(1.0, 10., TEXT_BRIGHT))
                    .child(
                        div()
                            .w(px(8.))
                            .h(px(8.))
                            .rounded_full()
                            .bg(rgb(LED_OFF))
                            .border_1()
                            .border_color(rgb(BORDER)),
                    ),
            );
        }
        slots = slots
            .child(div().flex_1())
            .child(lcd().h(px(16.)).text_size(px(9.)).child("Out 1 - Out 2"));

        let header = window_title("▥", "Mixer")
            .child(chip("Wide"))
            .child(div().flex_1())
            .child(super::close_ctl(
                "close-mixer",
                |d| d.show_mixer = false,
                cx,
            ));

        fl_window().size_full().child(header).child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .overflow_hidden()
                .child(strip_row)
                .child(slots),
        )
    }

    fn render_strip(&self, s: StripData, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let is_master = s.num == 0;
        let header_color = s
            .color
            .unwrap_or(if is_master { 0x5b646b } else { 0x4a535a });
        let t_idx = s.track;
        let w = if is_master {
            MASTER_STRIP_W
        } else {
            MIXER_STRIP_W
        };

        div()
            .id(SharedString::from(format!("strip-{}", s.num)))
            .flex_none()
            .w(px(w))
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(4.))
            .pb(px(4.))
            .rounded(px(2.))
            .bg(if s.selected {
                vgrad(0x55616a, 0x414b53)
            } else {
                vgrad(0x434c53, 0x363e44)
            })
            .border_1()
            .border_color(rgb(if s.selected { BEVEL } else { BORDER }))
            .when(is_master, |d| d.mr(px(6.)))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                this.selected_track = t_idx;
                cx.notify();
            }))
            // Number / color header.
            .child(
                div()
                    .flex_none()
                    .w_full()
                    .h(px(14.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(raised(header_color))
                    .text_size(px(9.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(TEXT_BRIGHT))
                    .child(SharedString::from(if is_master {
                        "M".to_string()
                    } else {
                        s.num.to_string()
                    })),
            )
            // Name.
            .child(
                div()
                    .flex_none()
                    .w_full()
                    .h(px(24.))
                    .px(px(2.))
                    .overflow_hidden()
                    .text_size(px(9.))
                    .line_height(px(11.))
                    .text_center()
                    .text_color(rgb(if s.track.is_some() || is_master {
                        TEXT
                    } else {
                        TEXT_DIM
                    }))
                    .child(SharedString::from(s.name)),
            )
            .child({
                let pan_knob = knob((s.pan + 1.0) / 2.0, 16., TEXT_BRIGHT);
                match t_idx {
                    Some(t) => pan_knob
                        .id(SharedString::from(format!("pan-{}", s.num)))
                        .cursor_col_resize()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, ev: &MouseDownEvent, _w, cx| {
                                cx.stop_propagation();
                                this.ui_drag = Some(UiDrag::Pan {
                                    track: t,
                                    start_x: f32::from(ev.position.x),
                                    orig: this.mix[t].pan,
                                });
                                cx.notify();
                            }),
                        )
                        .into_any_element(),
                    None => pan_knob.into_any_element(),
                }
            })
            // Meter + fader.
            .child({
                let row = div()
                    .id(SharedString::from(format!("fader-{}", s.num)))
                    .flex_1()
                    .flex()
                    .flex_row()
                    .gap(px(3.))
                    .py(px(4.))
                    .child(meter(s.level))
                    .child(fader(s.fader));
                if t_idx.is_some() || is_master {
                    let fader_val = s.fader;
                    row.cursor_ns_resize()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, ev: &MouseDownEvent, _w, cx| {
                                cx.stop_propagation();
                                this.ui_drag = Some(UiDrag::Fader {
                                    track: t_idx,
                                    start_y: f32::from(ev.position.y),
                                    orig: fader_val,
                                });
                                cx.notify();
                            }),
                        )
                        .into_any_element()
                } else {
                    row.into_any_element()
                }
            })
            // Enable LED (mutes the backing track).
            .child(led(
                SharedString::from(format!("strip-led-{}", s.num)),
                s.enabled,
                10.,
                cx.listener(move |this, _: &ClickEvent, _w, cx| {
                    cx.stop_propagation();
                    if let Some(t) = t_idx {
                        this.tracks[t].muted ^= true;
                    }
                    cx.notify();
                }),
            ))
    }
}
