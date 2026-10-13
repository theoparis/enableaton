mod browser;
mod channel_rack;
pub mod core;
mod device;
mod mixer;
mod piano_roll;
mod playlist;
mod toolbar;

use gpui::{
    AnyElement, ClickEvent, Context, IntoElement, MouseButton, MouseMoveEvent, MouseUpEvent,
    Render, SharedString, Window, div, prelude::*, px, rgb,
};

use crate::daw::Daw;
use crate::models::Instrument;
use crate::theme::*;
use core::button::tab_btn;

/// Gap between docked windows.
const DOCK_GAP: f32 = 3.0;

impl Render for Daw {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let toolbar = self.render_toolbar(cx);

        let mut workspace = div()
            .flex_1()
            .min_h(px(0.))
            .flex()
            .flex_row()
            .gap(px(DOCK_GAP))
            .p(px(DOCK_GAP))
            .overflow_hidden();

        if self.show_browser {
            workspace = workspace.child(self.render_browser(cx));
        }

        // Middle column: channel rack above the piano roll / channel settings.
        if self.show_channel_rack || self.show_piano_roll {
            let mut mid = div()
                .flex_none()
                .w(px(MID_COL_W))
                .h_full()
                .flex()
                .flex_col()
                .gap(px(DOCK_GAP));
            if self.show_channel_rack {
                let rack = self.render_channel_rack(cx);
                mid = mid.child(
                    div()
                        .flex()
                        .flex_col()
                        .when(self.show_piano_roll, |d| d.flex_none().max_h(px(300.)))
                        .when(!self.show_piano_roll, |d| d.flex_1())
                        .child(rack),
                );
            }
            if self.show_piano_roll {
                let lower: AnyElement =
                    match self.selected_track.map(|i| &self.tracks[i].instrument) {
                        Some(Instrument::DrumRack(_)) => {
                            self.render_drum_settings(cx).into_any_element()
                        }
                        Some(Instrument::Plugin(_)) if self.show_device => {
                            self.render_device_panel(cx).into_any_element()
                        }
                        _ => self.render_piano_roll(cx).into_any_element(),
                    };
                mid = mid.child(div().flex_1().min_h(px(0.)).child(lower));
            }
            workspace = workspace.child(mid);
        }

        workspace = workspace.child(self.render_playlist(cx));

        let mixer = self.show_mixer.then(|| {
            div()
                .flex_none()
                .h(px(MIXER_H))
                .px(px(DOCK_GAP))
                .pb(px(DOCK_GAP))
                .child(self.render_mixer(cx))
        });

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
            .child(toolbar)
            .child(workspace)
            .children(mixer)
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
