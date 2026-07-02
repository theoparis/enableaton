use gpui::{Div, FontWeight, IntoElement, SharedString, div, prelude::*, px, rgb};

use crate::theme::*;

pub const PANEL_HEADER_H: f32 = 28.0;

/// Header strip for a bottom panel (piano roll / drum rack / device).
/// Append title, chips and controls as children.
pub fn panel_header() -> Div {
    div()
        .flex_none()
        .h(px(PANEL_HEADER_H))
        .flex()
        .flex_row()
        .items_center()
        .px_3()
        .gap_3()
        .bg(rgb(SURFACE))
        .border_b_1()
        .border_color(rgb(BORDER))
}

/// Accent-colored bold panel title.
pub fn panel_title(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(ACCENT))
        .child(text.into())
}

/// Small bordered info pill ("1/16", "BAR 3", zoom %, …).
pub fn chip(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .px_1()
        .rounded_sm()
        .bg(rgb(SURFACE2))
        .border_1()
        .border_color(rgb(BORDER))
        .text_xs()
        .text_color(rgb(TEXT_DIM))
        .child(text.into())
}

/// Outer shell for a bottom panel: fixed height, column layout, top border.
pub fn panel_shell(height: f32) -> Div {
    div()
        .flex_none()
        .h(px(height))
        .flex()
        .flex_col()
        .bg(rgb(BG))
        .border_t_1()
        .border_color(rgb(BORDER))
}
