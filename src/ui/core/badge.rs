use gpui::{FontWeight, IntoElement, SharedString, div, prelude::*, px, rgb};

use crate::theme::*;

/// Small colored pill showing a 2-char instrument-type abbreviation.
pub fn sample_badge(label: impl Into<SharedString>, bg_color: u32) -> impl IntoElement {
    div()
        .flex_none()
        .px(px(3.))
        .rounded(px(2.))
        .bg(rgb(bg_color))
        .border_1()
        .border_color(rgb(BORDER))
        .text_size(px(9.))
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(TEXT_BRIGHT))
        .child(label.into())
}
