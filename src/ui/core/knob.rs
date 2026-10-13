use gpui::{Div, div, prelude::*, px, rgb};

use super::button::vgrad;
use crate::theme::*;

/// FL-style rotary knob. `value` is normalized 0..1 and maps to a 270° sweep
/// (7:30 → 4:30 o'clock). Returned as a `Div` so callers can attach handlers.
pub fn knob(value: f32, size: f32, accent: u32) -> Div {
    let angle = (-135.0 + 270.0 * value.clamp(0.0, 1.0)).to_radians();
    let dot = (size * 0.18).max(2.0);
    let r = size * 0.5 - dot * 0.5 - size * 0.12;
    let cx = size * 0.5 + r * angle.sin() - dot * 0.5;
    let cy = size * 0.5 - r * angle.cos() - dot * 0.5;

    div()
        .flex_none()
        .relative()
        .w(px(size))
        .h(px(size))
        .rounded_full()
        .bg(vgrad(0x6a747c, 0x3a4248))
        .border_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .absolute()
                .left(px(cx))
                .top(px(cy))
                .w(px(dot))
                .h(px(dot))
                .rounded_full()
                .bg(rgb(accent)),
        )
}
