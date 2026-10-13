use gpui::{
    App, Background, ClickEvent, FontWeight, IntoElement, SharedString, Window, div,
    linear_color_stop, linear_gradient, prelude::*, px, rgb,
};

use crate::theme::*;

/// Vertical two-stop gradient, top → bottom.
pub fn vgrad(top: u32, bottom: u32) -> Background {
    linear_gradient(
        180.,
        linear_color_stop(rgb(top), 0.),
        linear_color_stop(rgb(bottom), 1.),
    )
}

/// Raised FL-style button surface for the given base color.
pub fn raised(base: u32) -> Background {
    vgrad(lighten(base, 0.12), darken(base, 0.12))
}

/// Square toolbar button with a glyph. `active` lights it in `on_color`.
pub fn tool_btn(
    id: impl Into<SharedString>,
    glyph: impl Into<SharedString>,
    size: f32,
    active: bool,
    on_color: u32,
    handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .flex_none()
        .w(px(size))
        .h(px(size))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(3.))
        .bg(if active {
            raised(darken(on_color, 0.35))
        } else {
            raised(SURFACE2)
        })
        .border_1()
        .border_color(rgb(BORDER))
        .text_size(px(size * 0.5))
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(if active { on_color } else { TEXT }))
        .cursor_pointer()
        .hover(|s| s.border_color(rgb(BEVEL)))
        .on_click(handler)
        .child(glyph.into())
}

/// Small tab/selector button used in window title bars (pattern selection,
/// NOTES/DEVICE, …).
pub fn tab_btn(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    active: bool,
    handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .flex_none()
        .px(px(6.))
        .h(px(15.))
        .flex()
        .items_center()
        .rounded(px(2.))
        .text_size(px(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(if active { LCD_BG } else { TEXT }))
        .bg(if active {
            raised(ACCENT)
        } else {
            raised(SURFACE2)
        })
        .border_1()
        .border_color(rgb(BORDER))
        .cursor_pointer()
        .hover(|s| s.border_color(rgb(BEVEL)))
        .on_click(handler)
        .child(label.into())
}

/// 16×16 square toggle button (zoom +/-, …).
pub fn icon_btn(
    id: impl Into<SharedString>,
    glyph: impl Into<SharedString>,
    active: bool,
    handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    tool_btn(id, glyph, 16., active, ACCENT, handler)
}

/// Round status LED (FL's green channel/insert enable light).
pub fn led(
    id: impl Into<SharedString>,
    on: bool,
    size: f32,
    handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .flex_none()
        .w(px(size))
        .h(px(size))
        .rounded_full()
        .bg(if on {
            vgrad(lighten(LED_ON, 0.35), darken(LED_ON, 0.15))
        } else {
            vgrad(LED_OFF, darken(LED_OFF, 0.3))
        })
        .border_1()
        .border_color(rgb(BORDER))
        .cursor_pointer()
        .on_click(handler)
}
