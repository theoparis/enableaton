use gpui::{Div, FontWeight, IntoElement, SharedString, div, prelude::*, px, rgb};

use super::button::vgrad;
use crate::theme::*;

/// Docked FL-style window: rounded frame with a gradient title bar. Append
/// body content as children after `.child(window_title(...))`.
pub fn fl_window() -> Div {
    div()
        .flex()
        .flex_col()
        .overflow_hidden()
        .rounded(px(3.))
        .bg(rgb(SURFACE))
        .border_1()
        .border_color(rgb(BORDER))
}

/// Title bar strip: glyph + title, followed by any extra children
/// (tabs, chips, buttons). A trailing window-control cluster is added by
/// `window_controls()`.
pub fn window_title(glyph: &'static str, title: impl Into<SharedString>) -> Div {
    div()
        .flex_none()
        .h(px(WINDOW_TITLE_H))
        .flex()
        .flex_row()
        .items_center()
        .px(px(6.))
        .gap(px(6.))
        .bg(vgrad(TITLE_TOP, TITLE_BOTTOM))
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(ACCENT))
                .font_weight(FontWeight::BOLD)
                .child(glyph),
        )
        .child(
            div()
                .flex_none()
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(TEXT_BRIGHT))
                .child(title.into()),
        )
}

/// Decorative minimize/maximize/close cluster at the right of a title bar.
pub fn window_controls() -> impl IntoElement {
    div()
        .flex_none()
        .flex()
        .flex_row()
        .gap(px(3.))
        .children(["▁", "□", "✕"].map(|g| {
            div()
                .w(px(13.))
                .h(px(13.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(2.))
                .text_size(px(8.))
                .text_color(rgb(TEXT_DIM))
                .bg(rgb(darken(TITLE_BOTTOM, 0.2)))
                .child(g)
        }))
}

/// Small recessed info pill ("1/16", "BAR 3", zoom %, …).
pub fn chip(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .flex_none()
        .px(px(5.))
        .h(px(15.))
        .flex()
        .items_center()
        .rounded(px(2.))
        .bg(rgb(LCD_BG))
        .border_1()
        .border_color(rgb(BORDER))
        .text_size(px(10.))
        .text_color(rgb(LCD_TEXT))
        .child(text.into())
}

/// LCD readout box (tempo, song position, pattern selector).
pub fn lcd() -> Div {
    div()
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .px(px(6.))
        .rounded(px(2.))
        .bg(vgrad(LCD_BG, 0x20262a))
        .border_1()
        .border_color(rgb(BORDER))
        .text_color(rgb(LCD_TEXT))
}
