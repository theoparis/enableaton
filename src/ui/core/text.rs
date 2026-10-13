use gpui::{IntoElement, SharedString, div, prelude::*, px, rgb};

use crate::theme::TEXT_DIM;

/// Small dimmed label used for secondary text everywhere.
pub fn dim(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .flex_none()
        .text_size(px(10.))
        .text_color(rgb(TEXT_DIM))
        .child(text.into())
}
