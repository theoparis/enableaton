use vello_common::peniko::FontData;
use vello_hybrid::Scene;

mod arrangement;
mod layout;
mod primitives;
mod text;

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
	let layout = layout::Layout::new(
		width,
		height,
		arrangement::BASE_WIDTH,
		arrangement::BASE_HEIGHT,
	);
	arrangement::draw(scene, &layout, font);
}
