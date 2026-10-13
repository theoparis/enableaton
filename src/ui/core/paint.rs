use gpui::{Bounds, Pixels, Point, Window, fill, point, px, rgb, size};

/// Paint a solid rectangle at `(x, y)` (in px) relative to `origin`.
/// Decoration drawn this way has no layout node or hitbox, which keeps
/// per-frame cost flat for large grids and previews.
pub fn quad(
    window: &mut Window,
    origin: Point<Pixels>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: u32,
) {
    window.paint_quad(fill(
        Bounds::new(
            point(origin.x + px(x), origin.y + px(y)),
            size(px(w), px(h)),
        ),
        rgb(color),
    ));
}
