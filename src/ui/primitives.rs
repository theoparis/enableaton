use vello_common::kurbo::BezPath;
use vello_common::peniko::color::{AlphaColor, Srgb};
use vello_hybrid::Scene;

use crate::ui::layout::Layout;

pub fn fill_rect(
	scene: &mut Scene,
	layout: &Layout,
	x0: f64,
	y0: f64,
	x1: f64,
	y1: f64,
	color: AlphaColor<Srgb>,
) {
	scene.set_paint(color);
	scene.fill_rect(&layout.rect(x0, y0, x1, y1));
}

pub fn fill_viewport(
	scene: &mut Scene,
	layout: &Layout,
	color: AlphaColor<Srgb>,
) {
	scene.set_paint(color);
	scene.fill_rect(&layout.viewport_rect());
}

pub fn draw_triangle(
	scene: &mut Scene,
	layout: &Layout,
	x: f64,
	y: f64,
	w: f64,
	h: f64,
	color: AlphaColor<Srgb>,
) {
	let mut path = BezPath::new();
	let p0 = layout.point(x, y);
	let p1 = layout.point(x + w, y + h * 0.5);
	let p2 = layout.point(x, y + h);
	path.move_to((p0.x, p0.y));
	path.line_to((p1.x, p1.y));
	path.line_to((p2.x, p2.y));
	path.close_path();
	scene.set_paint(color);
	scene.fill_path(&path);
}

pub fn rgb(r: f32, g: f32, b: f32) -> AlphaColor<Srgb> {
	AlphaColor::<Srgb>::new([r, g, b, 1.0])
}

pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> AlphaColor<Srgb> {
	AlphaColor::<Srgb>::new([r, g, b, a])
}
