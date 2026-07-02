use gpui::{
    App, ClickEvent, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px, rgb,
};

use crate::theme::*;

/// Transport-bar button: icon plus optional text label.
pub fn transport_btn(
    icon: &'static str,
    label: &'static str,
    id: impl Into<SharedString>,
    active: bool,
    handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
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

/// Small tab/selector button used in panel headers (NOTES/DEVICE, pattern
/// selection, …).
pub fn tab_btn(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    active: bool,
    handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .px_2()
        .py_px()
        .rounded_sm()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(if active { ACCENT } else { TEXT_DIM }))
        .bg(rgb(if active { ACCENT_DIM } else { SURFACE2 }))
        .border_1()
        .border_color(rgb(if active { ACCENT } else { BORDER }))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(BORDER)))
        .on_click(handler)
        .child(label.into())
}

/// 18×18 square toggle button (mute buttons, zoom +/-, …).
pub fn icon_btn(
    id: impl Into<SharedString>,
    glyph: impl Into<SharedString>,
    active: bool,
    handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .flex_none()
        .w(px(18.))
        .h(px(18.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_sm()
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(if active { ACCENT } else { TEXT_DIM }))
        .bg(rgb(if active { ACCENT_DIM } else { SURFACE2 }))
        .cursor_pointer()
        .hover(|s| s.text_color(rgb(ACCENT)))
        .on_click(handler)
        .child(glyph.into())
}
