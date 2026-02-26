use vello_common::kurbo::BezPath;
use vello_hybrid::Scene;

use crate::ui::UiFont;
use crate::ui::layout::Layout;
use crate::ui::primitives::{
	draw_triangle, fill_rect, fill_viewport, rgb, rgba,
};
use crate::ui::text::draw_text;

pub const BASE_WIDTH: f64 = 820.0;
pub const BASE_HEIGHT: f64 = 742.0;
const TOP_BAR_HEIGHT: f64 = 42.0;

const LOCATOR_LEFT: f64 = 26.0;
const LOCATOR_VERSE_IN: f64 = 258.0;
const LOCATOR_VERSE_OUT: f64 = 287.0;

pub fn draw(scene: &mut Scene, layout: &Layout, font: &UiFont) {
	let view_w = layout.content_width();
	let view_h = layout.content_height();

	fill_viewport(scene, layout, rgb(0.16, 0.17, 0.19));
	fill_rect(
		scene,
		layout,
		0.0,
		0.0,
		view_w,
		view_h,
		rgb(0.16, 0.17, 0.19),
	);

	draw_grid(scene, layout, view_w, view_h);
	draw_ruler(scene, layout, font, view_w);
	draw_midi_lane(scene, layout, view_w);
	draw_audio_blocks(scene, layout, font);
	draw_locators(scene, layout, view_h);
	draw_top_bar(scene, layout, font);
}

fn draw_grid(scene: &mut Scene, layout: &Layout, view_w: f64, view_h: f64) {
	let minor = rgb(0.11, 0.12, 0.13);
	let major = rgb(0.17, 0.18, 0.2);

	let mut i = 0usize;
	loop {
		let x = 27.0 + i as f64 * 18.1;
		if x > view_w {
			break;
		}
		fill_rect(
			scene,
			layout,
			x,
			0.0,
			x + 3.0,
			view_h,
			if i % 4 == 0 { major } else { minor },
		);
		i += 1;
	}

	for y in [264.0, 350.0, 460.0, 630.0, 720.0] {
		fill_rect(
			scene,
			layout,
			0.0,
			y,
			view_w,
			y + 5.0,
			rgb(0.13, 0.14, 0.15),
		);
	}
}

fn draw_ruler(scene: &mut Scene, layout: &Layout, font: &UiFont, view_w: f64) {
	let header = rgb(0.20, 0.21, 0.23);
	let accent = rgba(0.81, 0.70, 0.52, 0.95);

	fill_rect(scene, layout, 27.0, 46.0, view_w, 78.0, header);
	fill_rect(
		scene,
		layout,
		27.0,
		46.0,
		view_w,
		48.5,
		rgb(0.78, 0.67, 0.49),
	);
	fill_rect(scene, layout, 27.0, 79.0, view_w, 122.0, header);

	draw_triangle(scene, layout, 29.0, 60.0, 28.0, 15.0, accent);
	draw_triangle(scene, layout, 30.0, 100.0, 24.0, 17.0, accent);
	draw_triangle(
		scene,
		layout,
		LOCATOR_VERSE_IN + 1.0,
		100.0,
		27.0,
		33.0,
		accent,
	);
	draw_triangle(scene, layout, LOCATOR_VERSE_OUT, 100.0, 29.0, 33.0, accent);

	let tick_y0 = 29.0;
	for (idx, beat) in [1, 5, 9, 13, 17, 21].iter().enumerate() {
		let x = 49.0 + idx as f64 * 134.0;
		fill_rect(scene, layout, x, tick_y0, x + 3.0, 49.0, accent);
		fill_rect(scene, layout, x + 3.0, 44.0, x + 35.0, 47.0, accent);
		draw_text(
			scene,
			layout,
			font,
			&beat.to_string(),
			x + 15.0,
			39.0,
			18.0,
			rgb(0.85, 0.75, 0.57),
		);
	}

	draw_text(
		scene,
		layout,
		font,
		"Intro",
		LOCATOR_LEFT + 37.0,
		112.0,
		42.0,
		rgb(0.85, 0.75, 0.57),
	);
	draw_text(
		scene,
		layout,
		font,
		"Verse A",
		LOCATOR_VERSE_OUT + 38.0,
		112.0,
		42.0,
		rgb(0.85, 0.75, 0.57),
	);
}

fn draw_midi_lane(scene: &mut Scene, layout: &Layout, view_w: f64) {
	fill_rect(
		scene,
		layout,
		27.0,
		124.0,
		view_w,
		233.0,
		rgb(0.35, 0.86, 0.81),
	);
	fill_rect(
		scene,
		layout,
		27.0,
		172.0,
		view_w,
		175.0,
		rgb(0.12, 0.49, 0.46),
	);
	fill_rect(
		scene,
		layout,
		27.0,
		231.0,
		view_w,
		234.0,
		rgb(0.12, 0.49, 0.46),
	);

	for i in 0..30 {
		let x = 30.0 + f64::from(i) * 26.5;
		fill_rect(scene, layout, x, 173.0, x + 2.0, 231.0, rgb(0.1, 0.53, 0.5));
	}

	for row in 0..3 {
		let y = 200.0 + f64::from(row) * 11.0;
		for i in 0..16 {
			let x0 = 32.0
				+ f64::from(i) * 65.0
				+ if i % 2 == 0 { 0.0 } else { 18.0 };
			fill_rect(
				scene,
				layout,
				x0,
				y,
				x0 + 42.0,
				y + 2.5,
				rgb(0.15, 0.19, 0.2),
			);
			fill_rect(
				scene,
				layout,
				x0 + 46.0,
				y,
				x0 + 57.0,
				y + 2.5,
				rgb(0.15, 0.19, 0.2),
			);
		}
	}
}

fn draw_audio_blocks(scene: &mut Scene, layout: &Layout, font: &UiFont) {
	let clip_color = rgb(0.98, 0.68, 0.17);
	let clip_dark = rgb(0.42, 0.30, 0.22);

	let clips = [
		(310.0, 360.0, 440.0, 640.0),
		(440.0, 360.0, 570.0, 640.0),
		(584.0, 360.0, 690.0, 640.0),
		(690.0, 360.0, 790.0, 640.0),
		(790.0, 360.0, 820.0, 640.0),
	];

	for (idx, &(x0, y0, x1, y1)) in clips.iter().enumerate() {
		fill_rect(scene, layout, x0, y0, x1, y1, clip_color);
		fill_rect(
			scene,
			layout,
			x0 + 2.0,
			y0 + 40.0,
			x1 - 2.0,
			y0 + 43.0,
			rgb(0.49, 0.34, 0.22),
		);

		for row in 0..6 {
			let y = y0 + 58.0 + row as f64 * 26.0;
			draw_smooth_wave_band(
				scene,
				layout,
				x0 + 2.0,
				x1 - 2.0,
				y,
				18.0,
				row,
				idx,
			);
			if row % 2 == 1 {
				fill_rect(
					scene,
					layout,
					x0 + 2.0,
					y,
					x1 - 2.0,
					y + 18.0,
					clip_dark,
				);
				draw_smooth_wave_band(
					scene,
					layout,
					x0 + 2.0,
					x1 - 2.0,
					y,
					18.0,
					row + 7,
					idx,
				);
			}
		}

		draw_text(
			scene,
			layout,
			font,
			"Lead Ta",
			x0 + 10.0,
			y0 + 30.0,
			18.0,
			rgb(0.12, 0.11, 0.10),
		);

		fill_rect(scene, layout, x0, y0, x0 + 2.5, y1, rgb(0.62, 0.45, 0.26));
		fill_rect(scene, layout, x1 - 2.5, y0, x1, y1, rgb(0.62, 0.45, 0.26));
	}
}

fn draw_locators(scene: &mut Scene, layout: &Layout, view_h: f64) {
	let locator = rgba(0.81, 0.70, 0.52, 0.95);
	fill_rect(
		scene,
		layout,
		LOCATOR_LEFT,
		0.0,
		LOCATOR_LEFT + 3.0,
		view_h,
		locator,
	);
	fill_rect(
		scene,
		layout,
		LOCATOR_VERSE_IN,
		82.0,
		LOCATOR_VERSE_IN + 3.0,
		view_h,
		locator,
	);
	fill_rect(
		scene,
		layout,
		LOCATOR_VERSE_OUT,
		82.0,
		LOCATOR_VERSE_OUT + 3.0,
		view_h,
		locator,
	);
}

fn draw_top_bar(scene: &mut Scene, layout: &Layout, font: &UiFont) {
	let bar_bg = rgb(0.14, 0.15, 0.16);
	let section_bg = rgb(0.19, 0.20, 0.22);
	let section_border = rgb(0.26, 0.27, 0.29);
	let text_color = rgb(0.84, 0.85, 0.87);
	let accent = rgb(0.74, 0.76, 0.80);

	let viewport = layout.viewport_rect();
	let top_bar_bottom = layout.point(0.0, TOP_BAR_HEIGHT).y;
	let divider = layout.length(1.5);
	scene.set_paint(bar_bg);
	scene.fill_rect(&vello_common::kurbo::Rect::new(
		viewport.x0,
		viewport.y0,
		viewport.x1,
		top_bar_bottom,
	));
	scene.set_paint(rgb(0.08, 0.09, 0.10));
	scene.fill_rect(&vello_common::kurbo::Rect::new(
		viewport.x0,
		top_bar_bottom - divider,
		viewport.x1,
		top_bar_bottom,
	));

	let sections = [
		(18.0, 8.0, 90.0, 34.0, "124.00"),    // bpm
		(98.0, 8.0, 146.0, 34.0, "4/4"),      // time signature
		(154.0, 8.0, 252.0, 34.0, "Met 1/4"), // metronome options
		(260.0, 8.0, 354.0, 34.0, "C Min"),   // key/scale
		(362.0, 8.0, 446.0, 34.0, "1.1.1"),   // beats/bar indicator
	];

	for (x0, y0, x1, y1, label) in sections {
		fill_rect(scene, layout, x0, y0, x1, y1, section_bg);
		fill_rect(scene, layout, x0, y0, x1, y0 + 1.5, section_border);
		fill_rect(scene, layout, x0, y1 - 1.5, x1, y1, section_border);
		draw_text(
			scene,
			layout,
			font,
			label,
			x0 + 10.0,
			y0 + 17.5,
			14.0,
			text_color,
		);
	}

	let button_y0 = 8.0;
	let button_y1 = 34.0;

	// play
	fill_rect(
		scene, layout, 474.0, button_y0, 508.0, button_y1, section_bg,
	);
	draw_triangle(scene, layout, 486.0, 14.0, 12.0, 14.0, accent);

	// pause
	fill_rect(
		scene, layout, 514.0, button_y0, 548.0, button_y1, section_bg,
	);
	fill_rect(scene, layout, 526.0, 14.0, 531.0, 28.0, accent);
	fill_rect(scene, layout, 535.0, 14.0, 540.0, 28.0, accent);

	// record
	fill_rect(
		scene, layout, 554.0, button_y0, 588.0, button_y1, section_bg,
	);
	fill_rect(
		scene,
		layout,
		566.0,
		14.0,
		576.0,
		24.0,
		rgb(0.88, 0.19, 0.23),
	);
}

fn draw_smooth_wave_band(
	scene: &mut Scene,
	layout: &Layout,
	x0: f64,
	x1: f64,
	y: f64,
	h: f64,
	row: usize,
	clip: usize,
) {
	let mid = y + h * 0.5;
	let sample_step = 3.5;
	let mut upper = Vec::new();
	let mut lower = Vec::new();

	let mut x = x0;
	while x <= x1 {
		let t = (x - x0) / (x1 - x0 + 0.001);
		let a = (t * std::f64::consts::TAU * 3.0
			+ row as f64 * 0.42
			+ clip as f64 * 0.81)
			.sin();
		let b = (t * std::f64::consts::TAU * 8.0
			+ row as f64 * 0.9
			+ clip as f64 * 0.37)
			.cos();
		let c = (t * std::f64::consts::TAU * 17.0 + clip as f64 * 0.55).sin();
		let envelope = (0.52 * a + 0.33 * b + 0.15 * c).abs();
		let amp = (1.1 + envelope * (h * 0.46)).min(h * 0.5 - 0.2);

		upper.push((x, mid - amp));
		lower.push((x, mid + amp));
		x += sample_step;
	}

	let mut path = BezPath::new();
	if let Some((ux, uy)) = upper.first().copied() {
		let p0 = layout.point(ux, uy);
		path.move_to((p0.x, p0.y));
		for (px, py) in upper.iter().skip(1).copied() {
			let p = layout.point(px, py);
			path.line_to((p.x, p.y));
		}
		for (px, py) in lower.iter().rev().copied() {
			let p = layout.point(px, py);
			path.line_to((p.x, p.y));
		}
		path.close_path();
		scene.set_paint(rgb(0.10, 0.10, 0.10));
		scene.fill_path(&path);
	}
}
