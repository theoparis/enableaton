use gpui::{FontWeight, IntoElement, SharedString, div, prelude::*, rgb};

/// Small colored pill showing a 2-char instrument-type abbreviation.
pub fn sample_badge(label: impl Into<SharedString>, bg_color: u32) -> impl IntoElement {
    div()
        .flex_none()
        .px_1()
        .rounded_sm()
        .bg(rgb(bg_color))
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(0xc8dff0))
        .child(label.into())
}
