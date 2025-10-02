use std::f32::consts::PI;

use vello::{
	AaConfig, Renderer, RendererOptions, Scene,
	kurbo::{Affine, BezPath, Rect, RoundedRect, Stroke},
	peniko::{Color, color::palette},
	util::{RenderContext, RenderSurface},
};
use wgpu::util::TextureBlitter;
use winit::{
	application::ApplicationHandler,
	event::WindowEvent,
	event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
	window::Window,
};

fn waveform_path(
	samples: &[f32],
	x_offset: f64,
	y_offset: f64,
	width: f64,
	height: f64,
) -> BezPath {
	let target_points = width as usize;
	let chunk_size = (samples.len() / target_points.max(1)).max(1);

	let mut path = BezPath::new();
	let mid_y = height / 2.0;
	let scale_y = height / 2.0;

	for (i, chunk) in samples.chunks(chunk_size).enumerate() {
		let avg = chunk.iter().copied().map(|v| v as f64).sum::<f64>()
			/ chunk.len() as f64;

		let x = (i as f64 / target_points as f64) * width;
		let y = mid_y - avg * scale_y;

		if i == 0 {
			path.move_to((x + x_offset, y + y_offset));
		} else {
			path.line_to((x + x_offset, y + y_offset));
		}
	}

	path
}

fn draw_waveform(
	scene: &mut vello::Scene,
	samples: &[f32],
	x_offset: f64,
	y_offset: f64,
	width: f64,
	height: f64,
) {
	let path = waveform_path(samples, x_offset, y_offset, width, height);

	scene.stroke(
		&Stroke::new(1.0),
		Affine::IDENTITY,
		Color::from_rgb8(255, 255, 255),
		None,
		&path,
	);
}

struct Clip {
	row: usize,
	start: f64,
	duration: f64,
	color: Color,
	data: Vec<f32>,
}

struct ClipView {
	window: Option<std::sync::Arc<Window>>,
	render_context: Option<RenderContext>,
	surface: Option<RenderSurface<'static>>,
	blitter: Option<TextureBlitter>,
	renderer: Option<Renderer>,
	scene: Scene,
	clips: Vec<Clip>,
}

impl ClipView {
	fn new() -> Self {
		let clips = vec![Clip {
			row: 0,
			start: 0.0,
			duration: 2.0,
			color: Color::from_rgb8(255, 140, 140),
			data: {
				let sample_rate = 48_000.0;
				let duration_seconds = 2.0;
				let amplitude = 0.5;
				let frequency = 440.0;

				let num_samples = (sample_rate * duration_seconds) as usize;
				let mut samples = Vec::with_capacity(num_samples);

				for i in 0..num_samples {
					let t = i as f32 / sample_rate; // Time in seconds for the current sample
					let value = amplitude * (2.0 * PI * frequency * t).sin();
					samples.push(value);
				}

				samples
			},
		}];

		Self {
			window: None,
			surface: None,
			render_context: None,
			renderer: None,
			blitter: None,
			scene: Scene::new(),
			clips,
		}
	}

	fn render_scene(&mut self) {
		self.scene.reset();

		let surface = self.surface.as_ref().unwrap();
		let width = surface.config.width as f64;
		let height = surface.config.height as f64;

		// Background
		self.scene.fill(
			vello::peniko::Fill::NonZero,
			Affine::IDENTITY,
			Color::from_rgb8(40, 40, 40),
			None,
			&Rect::new(0.0, 0.0, width, height),
		);

		// Render clips
		let row_height = 45.0;
		let col_width = 80.0;
		let padding = 2.0;
		let corner_radius = 3.0;

		for clip in &self.clips {
			let x = 50.0 + clip.start * col_width;
			let y = 50.0 + clip.row as f64 * row_height;
			let w = clip.duration * col_width - padding * 2.0;
			let h = row_height - padding * 2.0;

			// Clip background
			let clip_rect = RoundedRect::new(x, y, x + w, y + h, corner_radius);
			self.scene.fill(
				vello::peniko::Fill::NonZero,
				Affine::IDENTITY,
				clip.color,
				None,
				&clip_rect,
			);

			let sample_rate = 48_000.0;
			let start_index = ((clip.start / 1000.0) * sample_rate) as usize;
			let end_index = (((clip.start + clip.duration) / 1000.0)
				* sample_rate) as usize;
			let end_index = end_index.min(clip.data.len());
			let data = &clip.data[start_index..end_index];

			// Render waveform
			draw_waveform(
				&mut self.scene,
				data,
				x + padding,
				y + padding,
				w - padding * 2.0,
				h - padding * 2.0,
			);
		}

		// Timeline markers
		for i in 0..9 {
			let x = 50.0 + i as f64 * col_width;
			let marker_rect = Rect::new(x - 1.0, 20.0, x + 1.0, 30.0);
			self.scene.fill(
				vello::peniko::Fill::NonZero,
				Affine::IDENTITY,
				Color::from_rgb8(150, 150, 150),
				None,
				&marker_rect,
			);
		}
	}
}

impl ApplicationHandler for ClipView {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window_attrs = Window::default_attributes()
			.with_title("Ableton Clip View - Vello")
			.with_inner_size(winit::dpi::LogicalSize::new(800, 600));

		let window = std::sync::Arc::new(
			event_loop.create_window(window_attrs).unwrap(),
		);
		let mut render_context = RenderContext::new();

		let size = window.inner_size();

		let surface = pollster::block_on(render_context.create_surface(
			window.clone(),
			size.width,
			size.height,
			wgpu::PresentMode::AutoVsync,
		))
		.unwrap();

		let device_id =
			pollster::block_on(render_context.device(Some(&surface.surface)))
				.unwrap();
		let device = render_context.devices.get(device_id).unwrap();

		let renderer = Renderer::new(
			&device.device,
			RendererOptions {
				use_cpu: false,
				antialiasing_support: vello::AaSupport::all(),
				num_init_threads: None,
				..Default::default()
			},
		)
		.unwrap();

		let blitter = TextureBlitter::new(&device.device, surface.format);

		self.window = Some(window);
		self.surface = Some(surface);
		self.renderer = Some(renderer);
		self.render_context = Some(render_context);
		self.blitter = Some(blitter);

		self.render_scene();
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		_: winit::window::WindowId,
		event: WindowEvent,
	) {
		match event {
			WindowEvent::CloseRequested => {
				event_loop.exit();
			}
			WindowEvent::RedrawRequested => {
				let surface = self.surface.as_ref().unwrap();
				let render_context = self.render_context.as_mut().unwrap();
				let renderer = self.renderer.as_mut().unwrap();
				let blitter = self.blitter.as_mut().unwrap();
				let device_id = pollster::block_on(
					render_context.device(Some(&surface.surface)),
				)
				.unwrap();

				let device_handle =
					render_context.devices.get(device_id).unwrap();

				let width = surface.config.width;
				let height = surface.config.height;

				renderer
					.render_to_texture(
						&device_handle.device,
						&device_handle.queue,
						&self.scene,
						&surface.target_view,
						&vello::RenderParams {
							base_color: palette::css::BLACK,
							width,
							height,
							antialiasing_method: AaConfig::Msaa16,
						},
					)
					.unwrap();

				let surface_texture =
					surface.surface.get_current_texture().unwrap();

				let mut encoder = device_handle.device.create_command_encoder(
					&wgpu::CommandEncoderDescriptor {
						label: Some("blit_encoder"),
					},
				);

				blitter.copy(
					&device_handle.device,
					&mut encoder,
					&surface.target_view,
					&surface_texture
						.texture
						.create_view(&wgpu::TextureViewDescriptor::default()),
				);

				device_handle.queue.submit([encoder.finish()]);
				surface_texture.present();
				device_handle.device.poll(wgpu::PollType::Poll).unwrap();
			}
			WindowEvent::Resized(size) => {
				let render_context = self.render_context.as_ref().unwrap();
				let surface = self.surface.as_mut().unwrap();

				if size.width > 0 && size.height > 0 {
					render_context.resize_surface(
						surface,
						size.width,
						size.height,
					);
				}
			}
			_ => {}
		}
	}
}

fn main() {
	let event_loop = EventLoop::new().unwrap();
	event_loop.set_control_flow(ControlFlow::Wait);

	let mut app = ClipView::new();
	event_loop.run_app(&mut app).unwrap();
}
