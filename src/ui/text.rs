use skrifa::raw::FileRef;
use skrifa::{FontRef, MetadataProvider};
use vello_common::glyph::Glyph;
use vello_common::peniko::FontData;
use vello_common::peniko::color::{AlphaColor, Srgb};
use vello_hybrid::Scene;

use crate::ui::UiFont;
use crate::ui::layout::Layout;

pub fn draw_text(
	scene: &mut Scene,
	layout: &Layout,
	font: &UiFont,
	text: &str,
	x: f64,
	y: f64,
	size: f32,
	color: AlphaColor<Srgb>,
) {
	let scaled_size = layout.font_size(size);
	let (glyphs, _) = layout_glyphs(text, scaled_size, &font.font_data);
	let origin = layout.point(x, y);

	scene.set_paint(color);
	scene
		.glyph_run(&font.font_data)
		.font_size(scaled_size)
		.hint(true)
		.fill_glyphs(glyphs.into_iter().map(|mut glyph| {
			glyph.x += origin.x as f32;
			glyph.y += origin.y as f32;
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
