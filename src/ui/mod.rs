use vello_common::peniko::FontData;
use vello_hybrid::Scene;

mod arrangement;
mod layout;
mod primitives;
mod text;

pub use arrangement::TransportControl;

#[derive(Clone)]
pub struct UiFont {
	pub font_data: FontData,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TransportState {
	pub is_playing: bool,
	pub is_recording: bool,
	pub is_looping: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct AudioClipView {
	pub start_beats: f32,
	pub duration_beats: f32,
	pub frequency_hz: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct MidiNoteView {
	pub start_beats: f32,
	pub duration_beats: f32,
	pub midi_note: u8,
}

#[derive(Clone, Debug)]
pub struct MidiClipView {
	pub start_beats: f32,
	pub duration_beats: f32,
	pub notes: Vec<MidiNoteView>,
}

#[derive(Clone, Debug)]
pub struct ArrangementViewData {
	pub transport: TransportState,
	pub bpm: f32,
	pub playhead_beats: f32,
	pub clips: Vec<AudioClipView>,
	pub midi_clips: Vec<MidiClipView>,
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
	data: &ArrangementViewData,
) {
	let layout = layout::Layout::new(
		width,
		height,
		arrangement::BASE_WIDTH,
		arrangement::BASE_HEIGHT,
	);
	arrangement::draw(scene, &layout, font, data);
}

pub fn hit_test_transport_control(
	width: f64,
	height: f64,
	x: f64,
	y: f64,
) -> Option<TransportControl> {
	let layout = layout::Layout::new(
		width,
		height,
		arrangement::BASE_WIDTH,
		arrangement::BASE_HEIGHT,
	);
	arrangement::hit_test_transport(&layout, x, y)
}
