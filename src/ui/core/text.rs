use gpui::{IntoElement, SharedString, div, prelude::*, rgb};

use crate::theme::TEXT_DIM;

/// Small dimmed label used for secondary text everywhere.
pub fn dim(text: impl Into<SharedString>) -> impl IntoElement {
    div().text_xs().text_color(rgb(TEXT_DIM)).child(text.into())
}
