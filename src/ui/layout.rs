use vello_common::kurbo::{Point, Rect};

pub struct Layout {
	viewport_width: f64,
	viewport_height: f64,
	scale: f64,
	offset_x: f64,
	offset_y: f64,
	content_width: f64,
	content_height: f64,
}

impl Layout {
	pub fn new(
		viewport_width: f64,
		viewport_height: f64,
		content_width: f64,
		content_height: f64,
	) -> Self {
		let safe_content_width = content_width.max(1.0);
		let safe_content_height = content_height.max(1.0);
		let safe_viewport_width = viewport_width.max(1.0);
		let safe_viewport_height = viewport_height.max(1.0);
		// Match DAW-like behavior: keep a 1:1 baseline size and only scale down
		// when the window is too small, instead of scaling up on large windows.
		let scale = (safe_viewport_width / safe_content_width)
			.min(safe_viewport_height / safe_content_height)
			.min(1.0);
		Self {
			viewport_width: safe_viewport_width,
			viewport_height: safe_viewport_height,
			scale,
			// DAW UIs are usually anchored to the top-left, not centered.
			offset_x: 0.0,
			offset_y: 0.0,
			content_width: safe_viewport_width / scale,
			content_height: safe_viewport_height / scale,
		}
	}

	pub fn viewport_rect(&self) -> Rect {
		Rect::new(0.0, 0.0, self.viewport_width, self.viewport_height)
	}

	pub fn point(&self, x: f64, y: f64) -> Point {
		Point::new(
			self.offset_x + x * self.scale,
			self.offset_y + y * self.scale,
		)
	}

	pub fn rect(&self, x0: f64, y0: f64, x1: f64, y1: f64) -> Rect {
		Rect::new(
			self.offset_x + x0 * self.scale,
			self.offset_y + y0 * self.scale,
			self.offset_x + x1 * self.scale,
			self.offset_y + y1 * self.scale,
		)
	}

	pub fn font_size(&self, size: f32) -> f32 {
		size * self.scale as f32
	}

	pub fn length(&self, value: f64) -> f64 {
		value * self.scale
	}

	pub fn content_width(&self) -> f64 {
		self.content_width
	}

	pub fn content_height(&self) -> f64 {
		self.content_height
	}
}
