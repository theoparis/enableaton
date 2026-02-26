use vello_common::kurbo::{Point, Rect};

pub struct Layout {
	viewport_width: f64,
	viewport_height: f64,
	scale_x: f64,
	scale_y: f64,
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
		let scale_x = safe_viewport_width / safe_content_width;
		let scale_y = safe_viewport_height / safe_content_height;
		Self {
			viewport_width: safe_viewport_width,
			viewport_height: safe_viewport_height,
			scale_x,
			scale_y,
			// DAW UIs are usually anchored to the top-left, not centered.
			offset_x: 0.0,
			offset_y: 0.0,
			content_width: safe_content_width,
			content_height: safe_content_height,
		}
	}

	pub fn viewport_rect(&self) -> Rect {
		Rect::new(0.0, 0.0, self.viewport_width, self.viewport_height)
	}

	pub fn point(&self, x: f64, y: f64) -> Point {
		Point::new(
			self.offset_x + x * self.scale_x,
			self.offset_y + y * self.scale_y,
		)
	}

	pub fn rect(&self, x0: f64, y0: f64, x1: f64, y1: f64) -> Rect {
		Rect::new(
			self.offset_x + x0 * self.scale_x,
			self.offset_y + y0 * self.scale_y,
			self.offset_x + x1 * self.scale_x,
			self.offset_y + y1 * self.scale_y,
		)
	}

	pub fn font_size(&self, size: f32) -> f32 {
		size * self.scale_x.min(self.scale_y) as f32
	}

	pub fn length(&self, value: f64) -> f64 {
		value * self.scale_x.min(self.scale_y)
	}

	pub fn content_width(&self) -> f64 {
		self.content_width
	}

	pub fn content_height(&self) -> f64 {
		self.content_height
	}

	pub fn to_content_point(&self, x: f64, y: f64) -> (f64, f64) {
		(
			(x - self.offset_x) / self.scale_x,
			(y - self.offset_y) / self.scale_y,
		)
	}
}
