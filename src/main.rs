use std::{f32::consts::PI, sync::Arc};

use rodio::{OutputStream, Sink};
use wgpu::util::TextureBlitter;
use winit::{
	application::ApplicationHandler,
	error::EventLoopError,
	event::WindowEvent,
	event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
	window::Window,
};
use xilem::{
	Color, WidgetView, WindowOptions, Xilem,
	core::lens,
	masonry::kurbo::{BezPath, Point, QuadSpline, Shape as _, Stroke},
	style::{Padding, Style},
	view::{
		CrossAxisAlignment, MainAxisAlignment, flex_col, flex_row, label,
		text_button,
	},
};

use crate::clip::{Clip, ClipView};

mod clip;

struct AppState {
	bpm: f64,
	sample_rate: u32,
	clips: Vec<Arc<Clip>>,
	sink: Sink,
	stream: OutputStream,
}

impl AppState {
	pub fn new() -> Self {
		let stream = rodio::OutputStreamBuilder::open_default_stream().unwrap();
		let sample_rate = stream.config().sample_rate();

		let sink = rodio::Sink::connect_new(stream.mixer());
		sink.pause();

		Self {
			bpm: 120.0,
			clips: vec![
				Arc::new(Clip {
					track: 0,
					start: 0.0,
					duration: 2.0,
					color: Color::from_rgb8(200, 100, 100),
					data: (0..sample_rate as usize * 2)
						.map(|i| {
							(2.0 * PI * 440.0 * (i as f32 / sample_rate as f32))
								.sin() * 0.5
						})
						.collect(),
				}),
				Arc::new(Clip {
					track: 1,
					start: 1.0,
					duration: 2.0,
					color: Color::from_rgb8(100, 200, 100),
					data: (0..sample_rate as usize * 2)
						.map(|i| {
							(2.0 * PI * 660.0 * (i as f32 / sample_rate as f32))
								.sin() * 0.5
						})
						.collect(),
				}),
			],
			sample_rate,
			sink,
			stream,
		}
	}
}

fn clip_view(
	state: &mut AppState,
	index: usize,
) -> impl WidgetView<AppState> + use<> {
	ClipView::new(state.clips[index].clone(), state.bpm, state.sample_rate)
}

fn header(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
	flex_row((
		label(format!("BPM: {}", state.bpm)),
		text_button(
			if state.sink.is_paused() {
				"Play"
			} else {
				"Pause"
			},
			|state: &mut AppState| {
				if state.sink.is_paused() {
					state.sink.clear();
					for clip in &state.clips {
						let source = rodio::buffer::SamplesBuffer::new(
							1,
							state.sample_rate,
							clip.data.clone(),
						);
						state.sink.append(source);
					}
					state.sink.play();
				} else {
					state.sink.pause();
				}
			},
		),
	))
	.main_axis_alignment(MainAxisAlignment::Start)
	.cross_axis_alignment(CrossAxisAlignment::Center)
}

fn app_logic(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
	flex_col((
		header(state),
		(0..state.clips.len())
			.map(|i| clip_view(state, i))
			.collect::<Vec<_>>(),
	))
	.main_axis_alignment(MainAxisAlignment::Start)
	.cross_axis_alignment(CrossAxisAlignment::Start)
	.padding(Padding::all(10.0))
}

fn main() -> Result<(), EventLoopError> {
	tracing_subscriber::fmt::init();

	let app = Xilem::new_simple(
		AppState::new(),
		app_logic,
		WindowOptions::new("Ableton Clone"),
	);
	app.run_in(EventLoop::with_user_event())?;
	Ok(())
}
