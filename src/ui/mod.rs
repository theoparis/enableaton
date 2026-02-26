use skrifa::raw::FileRef;
use skrifa::{FontRef, MetadataProvider};
use vello_common::glyph::Glyph;
use vello_common::kurbo::{BezPath, Rect};
use vello_common::peniko::FontData;
use vello_common::peniko::color::{AlphaColor, Srgb};
use vello_hybrid::Scene;

#[derive(Clone)]
pub struct UiFont {
	pub font_data: FontData,
}

pub fn load_ui_font() -> UiFont {
	let font_data = FontData::new(
		include_bytes!("assets/GoogleSansCode.ttf").to_vec().into(),
		0,
	);

	UiFont { font_data }
}

pub fn draw_arrangement_view(
	scene: &mut Scene,
	width: f64,
	height: f64,
	font: &UiFont,
) {
	let sx = width / 820.0;
	let sy = height / 742.0;

	fill_rect(
		scene,
		scale_rect(0.0, 0.0, 820.0, 742.0, sx, sy),
		rgb(0.16, 0.17, 0.19),
	);

	draw_grid(scene, sx, sy);
	draw_ruler(scene, sx, sy, font);
	draw_midi_lane(scene, sx, sy);
	draw_audio_blocks(scene, sx, sy, font);
	draw_locators(scene, sx, sy);
}

const LOCATOR_LEFT: f64 = 26.0;
const LOCATOR_VERSE_IN: f64 = 258.0;
const LOCATOR_VERSE_OUT: f64 = 287.0;

fn draw_grid(scene: &mut Scene, sx: f64, sy: f64) {
	let minor = rgb(0.11, 0.12, 0.13);
	let major = rgb(0.17, 0.18, 0.2);

	for i in 0..45 {
		let x = 27.0 + f64::from(i) * 18.1;
		let line = scale_rect(x, 0.0, x + 3.0, 742.0, sx, sy);
		fill_rect(scene, line, if i % 4 == 0 { major } else { minor });
	}

	for y in [264.0, 350.0, 460.0, 630.0, 720.0] {
		fill_rect(
			scene,
			scale_rect(0.0, y, 820.0, y + 5.0, sx, sy),
			rgb(0.13, 0.14, 0.15),
		);
	}
}

fn draw_ruler(scene: &mut Scene, sx: f64, sy: f64, font: &UiFont) {
	let header = rgb(0.20, 0.21, 0.23);
	let accent = rgba(0.81, 0.70, 0.52, 0.95);

	fill_rect(scene, scale_rect(27.0, 46.0, 820.0, 78.0, sx, sy), header);
	fill_rect(
		scene,
		scale_rect(27.0, 46.0, 820.0, 48.5, sx, sy),
		rgb(0.78, 0.67, 0.49),
	);
	fill_rect(scene, scale_rect(27.0, 79.0, 820.0, 122.0, sx, sy), header);

	draw_triangle(scene, 29.0, 60.0, 28.0, 15.0, sx, sy, accent);
	draw_triangle(scene, 30.0, 100.0, 24.0, 17.0, sx, sy, accent);
	draw_triangle(
		scene,
		LOCATOR_VERSE_IN + 1.0,
		100.0,
		27.0,
		33.0,
		sx,
		sy,
		accent,
	);
	draw_triangle(scene, LOCATOR_VERSE_OUT, 100.0, 29.0, 33.0, sx, sy, accent);

	let tick_y0 = 29.0;
	for (idx, beat) in [1, 5, 9, 13, 17, 21].iter().enumerate() {
		let x = 49.0 + idx as f64 * 134.0;
		fill_rect(scene, scale_rect(x, tick_y0, x + 3.0, 49.0, sx, sy), accent);
		fill_rect(
			scene,
			scale_rect(x + 3.0, 44.0, x + 35.0, 47.0, sx, sy),
			accent,
		);
		draw_text(
			scene,
			font,
			&beat.to_string(),
			x + 15.0,
			39.0,
			18.0,
			rgb(0.85, 0.75, 0.57),
			sx,
			sy,
		);
	}

	draw_text(
		scene,
		font,
		"Intro",
		LOCATOR_LEFT + 37.0,
		112.0,
		42.0,
		rgb(0.85, 0.75, 0.57),
		sx,
		sy,
	);
	draw_text(
		scene,
		font,
		"Verse A",
		LOCATOR_VERSE_OUT + 38.0,
		112.0,
		42.0,
		rgb(0.85, 0.75, 0.57),
		sx,
		sy,
	);
}

fn draw_midi_lane(scene: &mut Scene, sx: f64, sy: f64) {
	fill_rect(
		scene,
		scale_rect(27.0, 124.0, 820.0, 233.0, sx, sy),
		rgb(0.35, 0.86, 0.81),
	);
	fill_rect(
		scene,
		scale_rect(27.0, 172.0, 820.0, 175.0, sx, sy),
		rgb(0.12, 0.49, 0.46),
	);
	fill_rect(
		scene,
		scale_rect(27.0, 231.0, 820.0, 234.0, sx, sy),
		rgb(0.12, 0.49, 0.46),
	);

	for i in 0..30 {
		let x = 30.0 + f64::from(i) * 26.5;
		fill_rect(
			scene,
			scale_rect(x, 173.0, x + 2.0, 231.0, sx, sy),
			rgb(0.1, 0.53, 0.5),
		);
	}

	for row in 0..3 {
		let y = 200.0 + f64::from(row) * 11.0;
		for i in 0..16 {
			let x0 = 32.0
				+ f64::from(i) * 65.0
				+ if i % 2 == 0 { 0.0 } else { 18.0 };
			fill_rect(
				scene,
				scale_rect(x0, y, x0 + 42.0, y + 2.5, sx, sy),
				rgb(0.15, 0.19, 0.2),
			);
			fill_rect(
				scene,
				scale_rect(x0 + 46.0, y, x0 + 57.0, y + 2.5, sx, sy),
				rgb(0.15, 0.19, 0.2),
			);
		}
	}
}

fn draw_audio_blocks(scene: &mut Scene, sx: f64, sy: f64, font: &UiFont) {
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
		fill_rect(scene, scale_rect(x0, y0, x1, y1, sx, sy), clip_color);
		fill_rect(
			scene,
			scale_rect(x0 + 2.0, y0 + 40.0, x1 - 2.0, y0 + 43.0, sx, sy),
			rgb(0.49, 0.34, 0.22),
		);

		for row in 0..6 {
			let y = y0 + 58.0 + row as f64 * 26.0;
			draw_smooth_wave_band(
				scene,
				x0 + 2.0,
				x1 - 2.0,
				y,
				18.0,
				row,
				idx,
				sx,
				sy,
			);
			if row % 2 == 1 {
				fill_rect(
					scene,
					scale_rect(x0 + 2.0, y, x1 - 2.0, y + 18.0, sx, sy),
					clip_dark,
				);
				draw_smooth_wave_band(
					scene,
					x0 + 2.0,
					x1 - 2.0,
					y,
					18.0,
					row + 7,
					idx,
					sx,
					sy,
				);
			}
		}

		draw_text(
			scene,
			font,
			"Lead Ta",
			x0 + 10.0,
			y0 + 30.0,
			18.0,
			rgb(0.12, 0.11, 0.10),
			sx,
			sy,
		);

		fill_rect(
			scene,
			scale_rect(x0, y0, x0 + 2.5, y1, sx, sy),
			rgb(0.62, 0.45, 0.26),
		);
		fill_rect(
			scene,
			scale_rect(x1 - 2.5, y0, x1, y1, sx, sy),
			rgb(0.62, 0.45, 0.26),
		);
	}
}

fn draw_locators(scene: &mut Scene, sx: f64, sy: f64) {
	let locator = rgba(0.81, 0.70, 0.52, 0.95);
	fill_rect(
		scene,
		scale_rect(LOCATOR_LEFT, 0.0, LOCATOR_LEFT + 3.0, 742.0, sx, sy),
		locator,
	);
	fill_rect(
		scene,
		scale_rect(
			LOCATOR_VERSE_IN,
			82.0,
			LOCATOR_VERSE_IN + 3.0,
			742.0,
			sx,
			sy,
		),
		locator,
	);
	fill_rect(
		scene,
		scale_rect(
			LOCATOR_VERSE_OUT,
			82.0,
			LOCATOR_VERSE_OUT + 3.0,
			742.0,
			sx,
			sy,
		),
		locator,
	);
}

fn draw_smooth_wave_band(
	scene: &mut Scene,
	x0: f64,
	x1: f64,
	y: f64,
	h: f64,
	row: usize,
	clip: usize,
	sx: f64,
	sy: f64,
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
		path.move_to((ux * sx, uy * sy));
		for (px, py) in upper.iter().skip(1).copied() {
			path.line_to((px * sx, py * sy));
		}
		for (px, py) in lower.iter().rev().copied() {
			path.line_to((px * sx, py * sy));
		}
		path.close_path();
		scene.set_paint(rgb(0.10, 0.10, 0.10));
		scene.fill_path(&path);
	}
}

fn draw_triangle(
	scene: &mut Scene,
	x: f64,
	y: f64,
	w: f64,
	h: f64,
	sx: f64,
	sy: f64,
	color: AlphaColor<Srgb>,
) {
	let mut path = BezPath::new();
	path.move_to((x * sx, y * sy));
	path.line_to(((x + w) * sx, (y + h * 0.5) * sy));
	path.line_to((x * sx, (y + h) * sy));
	path.close_path();
	scene.set_paint(color);
	scene.fill_path(&path);
}

fn draw_text(
	scene: &mut Scene,
	font: &UiFont,
	text: &str,
	x: f64,
	y: f64,
	size: f32,
	color: AlphaColor<Srgb>,
	sx: f64,
	sy: f64,
) {
	let scaled_size = size * sy as f32;
	let (glyphs, _) = layout_glyphs(text, scaled_size, &font.font_data);
	scene.set_paint(color);
	scene
		.glyph_run(&font.font_data)
		.font_size(scaled_size)
		.hint(true)
		.fill_glyphs(glyphs.into_iter().map(|mut glyph| {
			glyph.x += (x * sx) as f32;
			glyph.y += (y * sy) as f32;
			glyph
		}));
}

fn layout_glyphs(
	text: &str,
	font_size: f32,
	font: &FontData,
) -> (Vec<Glyph>, f32) {
	let font_ref = {
		let Ok(file_ref) = FileRef::new(font.data.as_ref()) else {
			return (Vec::new(), 0.0);
		};

		match file_ref {
			FileRef::Font(font) => font,
			FileRef::Collection(collection) => match collection.get(font.index)
			{
				Ok(font) => font,
				Err(_) => return (Vec::new(), 0.0),
			},
		}
	};

	layout_glyphs_from_ref(text, font_size, &font_ref)
}

fn layout_glyphs_from_ref(
	text: &str,
	font_size: f32,
	font_ref: &FontRef<'_>,
) -> (Vec<Glyph>, f32) {
	let size = skrifa::instance::Size::new(font_size);
	let axes = font_ref.axes();
	let variations: [(&str, f32); 0] = [];
	let loc = axes.location(&variations);
	let charmap = font_ref.charmap();
	let metrics = font_ref.metrics(size, &loc);
	let glyph_metrics = font_ref.glyph_metrics(size, &loc);

	let mut pen_x = 0.0f32;
	let mut pen_y = 0.0f32;

	let glyphs = text
		.chars()
		.filter_map(|ch| {
			if ch == '\n' {
				pen_x = 0.0;
				pen_y += metrics.ascent - metrics.descent + metrics.leading;
				return None;
			}

			let gid = charmap.map(ch).unwrap_or_default();
			let advance =
				glyph_metrics.advance_width(gid).unwrap_or(font_size * 0.45);
			let glyph = Glyph {
				id: gid.to_u32(),
				x: pen_x,
				y: pen_y,
			};
			pen_x += advance;
			Some(glyph)
		})
		.collect::<Vec<_>>();

	(glyphs, metrics.ascent)
}

fn fill_rect(scene: &mut Scene, rect: Rect, color: AlphaColor<Srgb>) {
	scene.set_paint(color);
	scene.fill_rect(&rect);
}

fn scale_rect(x0: f64, y0: f64, x1: f64, y1: f64, sx: f64, sy: f64) -> Rect {
	Rect::new(x0 * sx, y0 * sy, x1 * sx, y1 * sy)
}

fn rgb(r: f32, g: f32, b: f32) -> AlphaColor<Srgb> {
	AlphaColor::<Srgb>::new([r, g, b, 1.0])
}

fn rgba(r: f32, g: f32, b: f32, a: f32) -> AlphaColor<Srgb> {
	AlphaColor::<Srgb>::new([r, g, b, a])
}
