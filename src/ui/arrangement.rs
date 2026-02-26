use vello_common::kurbo::BezPath;
use vello_hybrid::Scene;

use crate::ui::UiFont;
use crate::ui::layout::Layout;
use crate::ui::primitives::{
	draw_triangle, fill_rect, fill_viewport, rgb, rgba,
};
use crate::ui::text::draw_text;
use crate::ui::{
	ArrangementViewData, AudioClipView, MidiClipView, TransportState,
};

pub const BASE_WIDTH: f64 = 820.0;
pub const BASE_HEIGHT: f64 = 742.0;
const TOP_BAR_HEIGHT: f64 = 42.0;
const TIMELINE_ORIGIN_X: f64 = 49.0;
const BEAT_PIXELS: f64 = 33.5;

const LOCATOR_LEFT: f64 = 26.0;
const LOCATOR_VERSE_IN: f64 = 258.0;
const LOCATOR_VERSE_OUT: f64 = 287.0;

#[derive(Clone, Copy, Debug)]
pub enum TransportControl {
	Bpm,
	Play,
	Pause,
	Stop,
	Record,
	ToggleLoop,
}

pub fn hit_test_transport(
	layout: &Layout,
	x: f64,
	y: f64,
) -> Option<TransportControl> {
	let (cx, cy) = layout.to_content_point(x, y);
	if !(0.0..=TOP_BAR_HEIGHT).contains(&cy) {
		return None;
	}

	if contains(cx, cy, 18.0, 8.0, 90.0, 34.0) {
		return Some(TransportControl::Bpm);
	}
	if contains(cx, cy, 538.0, 8.0, 572.0, 34.0) {
		return Some(TransportControl::Play);
	}
	if contains(cx, cy, 578.0, 8.0, 612.0, 34.0) {
		return Some(TransportControl::Pause);
	}
	if contains(cx, cy, 618.0, 8.0, 652.0, 34.0) {
		return Some(TransportControl::Stop);
	}
	if contains(cx, cy, 658.0, 8.0, 692.0, 34.0) {
		return Some(TransportControl::Record);
	}
	if contains(cx, cy, 454.0, 8.0, 530.0, 34.0) {
		return Some(TransportControl::ToggleLoop);
	}

	None
}

pub fn draw(
	scene: &mut Scene,
	layout: &Layout,
	font: &UiFont,
	data: &ArrangementViewData,
) {
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
	draw_midi_lane(scene, layout, view_w, &data.midi_clips);
	draw_audio_blocks(scene, layout, font, &data.clips, data.bpm);
	draw_locators(scene, layout, view_h);
	draw_playhead(scene, layout, view_h, data.playhead_beats);
	draw_top_bar(
		scene,
		layout,
		font,
		data.transport,
		data.bpm,
		data.playhead_beats,
	);
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

fn draw_midi_lane(
	scene: &mut Scene,
	layout: &Layout,
	view_w: f64,
	midi_clips: &[MidiClipView],
) {
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

	for i in 0..60 {
		let x = 30.0 + f64::from(i) * 13.25;
		fill_rect(scene, layout, x, 173.0, x + 2.0, 231.0, rgb(0.1, 0.53, 0.5));
	}

	for row in 0..4 {
		let y = 178.0 + f64::from(row) * 13.0;
		fill_rect(
			scene,
			layout,
			27.0,
			y,
			view_w,
			y + 1.0,
			rgb(0.1, 0.44, 0.42),
		);
	}

	for (clip_idx, clip) in midi_clips.iter().enumerate() {
		let x0 = TIMELINE_ORIGIN_X + f64::from(clip.start_beats) * BEAT_PIXELS;
		let x1 = x0 + (f64::from(clip.duration_beats) * BEAT_PIXELS).max(32.0);
		fill_rect(scene, layout, x0, 172.0, x1, 231.0, rgb(0.41, 0.90, 0.85));
		fill_rect(scene, layout, x0, 172.0, x1, 175.0, rgb(0.20, 0.70, 0.66));

		for note in &clip.notes {
			let nx0 = x0 + f64::from(note.start_beats) * BEAT_PIXELS;
			let nx1 =
				nx0 + (f64::from(note.duration_beats) * BEAT_PIXELS).max(4.0);
			let pitch_row = (note.midi_note.saturating_sub(60) % 12) as f64;
			let y = 228.0 - pitch_row * 4.4;
			fill_rect(
				scene,
				layout,
				nx0,
				y,
				nx1,
				y + 3.2,
				if clip_idx % 2 == 0 {
					rgb(0.14, 0.21, 0.22)
				} else {
					rgb(0.11, 0.18, 0.19)
				},
			);
		}
	}
}

fn draw_audio_blocks(
	scene: &mut Scene,
	layout: &Layout,
	font: &UiFont,
	clips: &[AudioClipView],
	bpm: f32,
) {
	let clip_color = rgb(0.98, 0.68, 0.17);
	let clip_dark = rgb(0.42, 0.30, 0.22);
	let lane_top = 360.0;
	let row_height = 90.0;
	let row_gap = 10.0;
	let secs_per_beat = 60.0 / f64::from(bpm);

	for (idx, clip) in clips.iter().enumerate() {
		let row = idx % 3;
		let y0 = lane_top + row as f64 * (row_height + row_gap);
		let y1 = y0 + row_height;
		let x0 = TIMELINE_ORIGIN_X + f64::from(clip.start_beats) * BEAT_PIXELS;
		let clip_width =
			(f64::from(clip.duration_beats) * BEAT_PIXELS).max(28.0);
		let x1 = x0 + clip_width;

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
			&format!(
				"{:.0}Hz {:.2}b ({:.2}s)",
				clip.frequency_hz,
				clip.duration_beats,
				f64::from(clip.duration_beats) * secs_per_beat
			),
			x0 + 10.0,
			y0 + 30.0,
			14.0,
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

fn draw_top_bar(
	scene: &mut Scene,
	layout: &Layout,
	font: &UiFont,
	transport: TransportState,
	bpm: f32,
	playhead_beats: f32,
) {
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

	let beat_pos = f64::from(playhead_beats);
	let beat_number = beat_pos.floor() as i32 + 1;
	let bar = ((beat_number - 1) / 4) + 1;
	let beat_in_bar = ((beat_number - 1) % 4) + 1;
	let sub = ((beat_pos.fract() * 4.0).floor() as i32 + 1).clamp(1, 4);
	let beat_indicator = format!("{bar}.{beat_in_bar}.{sub}");
	let bpm_label = format!("{bpm:.2}");
	let loop_label = if transport.is_looping {
		"Loop On"
	} else {
		"Loop Off"
	};

	let sections = [
		(18.0, 8.0, 90.0, 34.0, bpm_label.as_str()), // bpm
		(98.0, 8.0, 146.0, 34.0, "4/4"),             // time signature
		(154.0, 8.0, 252.0, 34.0, "Met 1/4"),        // metronome options
		(260.0, 8.0, 354.0, 34.0, "C Min"),          // key/scale
		(362.0, 8.0, 446.0, 34.0, beat_indicator.as_str()), // beats/bar indicator
		(454.0, 8.0, 530.0, 34.0, loop_label),       // loop toggle
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
	let play_bg = if transport.is_playing {
		rgb(0.22, 0.32, 0.26)
	} else {
		section_bg
	};
	fill_rect(scene, layout, 538.0, button_y0, 572.0, button_y1, play_bg);
	draw_triangle(scene, layout, 550.0, 14.0, 12.0, 14.0, accent);

	// pause
	fill_rect(
		scene, layout, 578.0, button_y0, 612.0, button_y1, section_bg,
	);
	fill_rect(scene, layout, 590.0, 14.0, 595.0, 28.0, accent);
	fill_rect(scene, layout, 599.0, 14.0, 604.0, 28.0, accent);

	// stop
	fill_rect(
		scene, layout, 618.0, button_y0, 652.0, button_y1, section_bg,
	);
	fill_rect(scene, layout, 631.0, 15.0, 640.0, 24.0, accent);

	// record
	let record_bg = if transport.is_recording {
		rgb(0.34, 0.19, 0.20)
	} else {
		section_bg
	};
	fill_rect(scene, layout, 658.0, button_y0, 692.0, button_y1, record_bg);
	fill_rect(
		scene,
		layout,
		670.0,
		14.0,
		680.0,
		24.0,
		rgb(0.88, 0.19, 0.23),
	);
}

fn draw_playhead(
	scene: &mut Scene,
	layout: &Layout,
	view_h: f64,
	playhead_beats: f32,
) {
	let x = TIMELINE_ORIGIN_X + f64::from(playhead_beats) * BEAT_PIXELS;
	fill_rect(
		scene,
		layout,
		x,
		TOP_BAR_HEIGHT,
		x + 2.0,
		view_h,
		rgba(0.91, 0.84, 0.67, 0.95),
	);
}

fn contains(x: f64, y: f64, x0: f64, y0: f64, x1: f64, y1: f64) -> bool {
	x >= x0 && x <= x1 && y >= y0 && y <= y1
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
