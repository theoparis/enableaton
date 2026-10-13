mod browser;
mod channel_rack;
pub mod core;
mod device;
mod mixer;
mod panes;
mod piano_roll;
mod playlist;
mod toolbar;

pub use panes::{Panes, Ticker};

use gpui::{
    ClickEvent, Context, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Render, SharedString, StyleRefinement, Window, deferred, div, prelude::*, px, rgb,
};

use crate::daw::Daw;
use crate::models::{Instrument, PaneEdge, UiDrag};
use crate::theme::*;
use core::button::tab_btn;
use core::panel::window_controls;

/// Thickness of the draggable divider between docked windows.
const DIVIDER: f32 = 4.0;
/// Outer padding of the workspace.
const DOCK_PAD: f32 = 3.0;

/// Close button for a docked window; `hide` clears its visibility flag.
pub(super) fn close_ctl(
    id: &'static str,
    hide: fn(&mut Daw),
    cx: &mut Context<Daw>,
) -> impl IntoElement {
    window_controls(
        id,
        cx.listener(move |this, _: &ClickEvent, _w, cx| {
            hide(this);
            cx.notify();
        }),
    )
}

impl Daw {
    fn pane_size(&self, edge: PaneEdge) -> f32 {
        match edge {
            PaneEdge::Browser => self.browser_w,
            PaneEdge::MidColumn => self.mid_w,
            PaneEdge::Rack => self.rack_h,
            PaneEdge::Mixer => self.mixer_h,
        }
    }

    /// Draggable divider. `vertical` dividers (a vertical bar) resize
    /// horizontally; the others resize vertically.
    fn divider(&self, edge: PaneEdge, vertical: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let base = div()
            .id(SharedString::from(format!("divider-{edge:?}")))
            .flex_none()
            .hover(|s| s.bg(rgb(ACCENT_DIM)));
        let base = if vertical {
            base.w(px(DIVIDER)).h_full().cursor_col_resize()
        } else {
            base.h(px(DIVIDER)).w_full().cursor_row_resize()
        };
        base.on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, ev: &MouseDownEvent, _w, cx| {
                cx.stop_propagation();
                let pos = if vertical {
                    ev.position.x
                } else {
                    ev.position.y
                };
                this.ui_drag = Some(UiDrag::Pane {
                    edge,
                    start: f32::from(pos),
                    orig: this.pane_size(edge),
                });
                cx.notify();
            }),
        )
    }
}

impl Render for Daw {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let panes = match &self.panes {
            Some(p) => p.clone(),
            None => {
                let p = Panes::new(
                    &cx.entity(),
                    self.ticker.clone(),
                    self.meter_clock.clone(),
                    cx,
                );
                self.panes = Some(p.clone());
                p
            }
        };
        let full = || StyleRefinement::default().size_full();

        let mut workspace = div()
            .flex_1()
            .min_h(px(0.))
            .flex()
            .flex_row()
            .p(px(DOCK_PAD))
            .overflow_hidden();

        if self.show_browser {
            let style = StyleRefinement::default()
                .flex_none()
                .w(px(self.browser_w))
                .h_full();
            workspace = workspace
                .child(panes.browser.clone().cached(style))
                .child(self.divider(PaneEdge::Browser, true, cx));
        }

        // Middle column: channel rack above the piano roll / channel settings.
        if self.show_channel_rack || self.show_piano_roll {
            let mut mid = div()
                .flex_none()
                .w(px(self.mid_w))
                .h_full()
                .flex()
                .flex_col();
            if self.show_channel_rack {
                let both = self.show_piano_roll;
                mid = mid.child(
                    div()
                        .flex()
                        .flex_col()
                        .min_h(px(0.))
                        .when(both, |d| d.flex_none().h(px(self.rack_h)))
                        .when(!both, |d| d.flex_1())
                        .child(panes.rack.clone().cached(full())),
                );
                if both {
                    mid = mid.child(self.divider(PaneEdge::Rack, false, cx));
                }
            }
            if self.show_piano_roll {
                mid = mid.child(
                    div()
                        .flex_1()
                        .min_h(px(0.))
                        .relative()
                        .child(panes.lower.clone().cached(full()))
                        .child(panes.piano_roll_overlay.clone()),
                );
            }
            workspace = workspace
                .child(mid)
                .child(self.divider(PaneEdge::MidColumn, true, cx));
        }

        if self.show_playlist {
            workspace = workspace.child(
                div()
                    .flex_1()
                    .h_full()
                    .relative()
                    .child(panes.playlist.clone().cached(full()))
                    .child(panes.playlist_overlay.clone()),
            );
        } else {
            workspace = workspace.child(div().flex_1());
        }

        let mixer = self.show_mixer.then(|| {
            div()
                .flex_none()
                .flex()
                .flex_col()
                .h(px(self.mixer_h + DIVIDER))
                .px(px(DOCK_PAD))
                .pb(px(DOCK_PAD))
                .child(self.divider(PaneEdge::Mixer, false, cx))
                .child(
                    div()
                        .flex_1()
                        .min_h(px(0.))
                        .child(panes.mixer.clone().cached(full())),
                )
        });

        let backdrop = self.open_menu.is_some().then(|| {
            deferred(
                div()
                    .id("menu-backdrop")
                    .occlude()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseDownEvent, _w, cx| {
                            this.open_menu = None;
                            cx.notify();
                        }),
                    ),
            )
            .priority(1)
        });

        div()
            .id("root")
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(BG))
            // Global mouse handlers for drags (the pointer can leave the
            // control while dragging).
            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, _w, cx| {
                if ev.pressed_button != Some(MouseButton::Left) {
                    return;
                }
                this.apply_ui_drag(f32::from(ev.position.x), f32::from(ev.position.y), cx);
                if let Some((start_y, start_bpm)) = this.bpm_drag_origin {
                    let delta = ((start_y - f32::from(ev.position.y)) / 1.5) as i32;
                    this.bpm = (start_bpm as i32 + delta).clamp(20, 300) as u32;
                    cx.notify();
                }
                // Device panel parameter drag: full bar height = full param range.
                if let Some(drag) = this.param_drag.clone()
                    && let Instrument::Plugin(p) = &this.tracks[drag.track_idx].instrument
                {
                    let spec = &p.params()[drag.param_idx];
                    let delta_y = drag.start_y - f32::from(ev.position.y);
                    let new_val = drag.start_val + delta_y / 120.0 * (spec.max - spec.min);
                    p.set_param(drag.param_idx, new_val);
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _w, _cx| {
                    this.bpm_drag_origin = None;
                    this.param_drag = None;
                    this.ui_drag = None;
                }),
            )
            .child(panes.toolbar.clone())
            .child(workspace)
            .children(mixer)
            .children(backdrop)
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
