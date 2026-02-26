use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Scene};
use wgpu::{
	Adapter, Device, Features, Instance, Limits, Queue, Surface,
	SurfaceConfiguration, SurfaceTarget, TextureFormat,
};
use winit::{
	application::ApplicationHandler,
	event::WindowEvent,
	event_loop::{ActiveEventLoop, EventLoop},
	window::{Window, WindowAttributes, WindowId},
};

mod ui;

#[derive(Debug)]
struct RenderContext {
	instance: Instance,
	devices: Vec<DeviceHandle>,
}

#[derive(Debug)]
struct DeviceHandle {
	adapter: Adapter,
	device: Device,
	queue: Queue,
}

#[derive(Debug)]
struct RenderSurface<'s> {
	surface: Surface<'s>,
	config: SurfaceConfiguration,
	dev_id: usize,
}

impl RenderContext {
	fn new() -> Self {
		let instance = Instance::new(&wgpu::InstanceDescriptor::default());
		Self {
			instance,
			devices: Vec::new(),
		}
	}

	async fn create_surface<'w>(
		&mut self,
		window: impl Into<SurfaceTarget<'w>>,
		width: u32,
		height: u32,
		present_mode: wgpu::PresentMode,
		format: TextureFormat,
	) -> RenderSurface<'w> {
		self.create_render_surface(
			self.instance
				.create_surface(window.into())
				.expect("failed to create surface"),
			width,
			height,
			present_mode,
			format,
		)
		.await
	}

	async fn create_render_surface<'w>(
		&mut self,
		surface: Surface<'w>,
		width: u32,
		height: u32,
		present_mode: wgpu::PresentMode,
		format: TextureFormat,
	) -> RenderSurface<'w> {
		let dev_id = self
			.device(Some(&surface))
			.await
			.expect("no compatible device");

		let config = SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format,
			width,
			height,
			present_mode,
			desired_maximum_frame_latency: 2,
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			view_formats: vec![],
		};

		let surface = RenderSurface {
			surface,
			config,
			dev_id,
		};
		self.configure_surface(&surface);
		surface
	}

	fn resize_surface(
		&self,
		surface: &mut RenderSurface<'_>,
		width: u32,
		height: u32,
	) {
		surface.config.width = width.max(1);
		surface.config.height = height.max(1);
		self.configure_surface(surface);
	}

	fn configure_surface(&self, surface: &RenderSurface<'_>) {
		surface
			.surface
			.configure(&self.devices[surface.dev_id].device, &surface.config);
	}

	async fn device(
		&mut self,
		compatible_surface: Option<&Surface<'_>>,
	) -> Option<usize> {
		let compatible = match compatible_surface {
			Some(surface) => self
				.devices
				.iter()
				.enumerate()
				.find(|(_, handle)| {
					handle.adapter.is_surface_supported(surface)
				})
				.map(|(idx, _)| idx),
			None => (!self.devices.is_empty()).then_some(0),
		};

		if compatible.is_some() {
			compatible
		} else {
			self.new_device(compatible_surface).await
		}
	}

	async fn new_device(
		&mut self,
		compatible_surface: Option<&Surface<'_>>,
	) -> Option<usize> {
		let adapter = wgpu::util::initialize_adapter_from_env_or_default(
			&self.instance,
			compatible_surface,
		)
		.await
		.ok()?;

		let (device, queue) = adapter
			.request_device(&wgpu::DeviceDescriptor {
				label: Some("wgpu device"),
				required_features: Features::empty(),
				required_limits: Limits::default(),
				..Default::default()
			})
			.await
			.ok()?;

		self.devices.push(DeviceHandle {
			adapter,
			device,
			queue,
		});

		Some(self.devices.len() - 1)
	}
}

fn create_renderer(
	context: &RenderContext,
	surface: &RenderSurface<'_>,
) -> Renderer {
	Renderer::new(
		&context.devices[surface.dev_id].device,
		&RenderTargetConfig {
			format: surface.config.format,
			width: surface.config.width,
			height: surface.config.height,
		},
	)
}

#[derive(Debug)]
enum RenderState {
	Active {
		surface: Box<RenderSurface<'static>>,
		window: Box<dyn Window>,
	},
	Suspended,
}

struct ApplicationState {
	context: RenderContext,
	renderers: Vec<Option<Renderer>>,
	render_state: RenderState,
	scene: Scene,
	font: ui::UiFont,
}

impl Default for ApplicationState {
	fn default() -> Self {
		Self {
			context: RenderContext::new(),
			renderers: Vec::new(),
			render_state: RenderState::Suspended,
			scene: Scene::new(820, 742),
			font: ui::load_ui_font(),
		}
	}
}

impl ApplicationHandler for ApplicationState {
	fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
		if matches!(self.render_state, RenderState::Active { .. }) {
			return;
		}

		let window = event_loop
			.create_window(
				WindowAttributes::default()
					.with_title("Ableton Clone in Rust")
					.with_surface_size(winit::dpi::PhysicalSize::new(820, 742)),
			)
			.expect("create window");

		let size = window.surface_size();
		let window_ref: &'static dyn Window = unsafe {
			std::mem::transmute::<&dyn Window, &'static dyn Window>(
				window.as_ref(),
			)
		};
		let surface = pollster::block_on(self.context.create_surface(
			window_ref,
			size.width,
			size.height,
			wgpu::PresentMode::AutoVsync,
			wgpu::TextureFormat::Bgra8Unorm,
		));

		self.renderers
			.resize_with(self.context.devices.len(), || None);
		self.renderers[surface.dev_id]
			.get_or_insert_with(|| create_renderer(&self.context, &surface));

		self.scene = Scene::new(
			u16::try_from(size.width).unwrap_or(u16::MAX),
			u16::try_from(size.height).unwrap_or(u16::MAX),
		);

		self.render_state = RenderState::Active {
			surface: Box::new(surface),
			window,
		};

		let RenderState::Active { surface, .. } = &mut self.render_state else {
			return;
		};

		ui::draw_arrangement_view(
			&mut self.scene,
			f64::from(surface.config.width),
			f64::from(surface.config.height),
			&self.font,
		);
	}

	fn window_event(
		&mut self,
		event_loop: &dyn ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		let RenderState::Active { surface, window } = &mut self.render_state
		else {
			return;
		};

		if window.id() != window_id {
			return;
		}

		match event {
			WindowEvent::CloseRequested => event_loop.exit(),
			WindowEvent::SurfaceResized(size) => {
				self.context.resize_surface(
					surface,
					size.width.max(1),
					size.height.max(1),
				);
				self.scene = Scene::new(
					u16::try_from(size.width.max(1)).unwrap_or(u16::MAX),
					u16::try_from(size.height.max(1)).unwrap_or(u16::MAX),
				);
				self.scene.reset();
				ui::draw_arrangement_view(
					&mut self.scene,
					f64::from(surface.config.width),
					f64::from(surface.config.height),
					&self.font,
				);
				window.request_redraw();
			}
			WindowEvent::RedrawRequested => {
				let Some(renderer) = self.renderers[surface.dev_id].as_mut()
				else {
					return;
				};
				let device_handle = &self.context.devices[surface.dev_id];

				let Ok(surface_texture) = surface.surface.get_current_texture()
				else {
					self.context.resize_surface(
						surface,
						surface.config.width,
						surface.config.height,
					);
					return;
				};
				let texture_view = surface_texture
					.texture
					.create_view(&wgpu::TextureViewDescriptor::default());

				let mut encoder = device_handle.device.create_command_encoder(
					&wgpu::CommandEncoderDescriptor {
						label: Some("vello-ui-render"),
					},
				);

				let render_size = RenderSize {
					width: surface.config.width,
					height: surface.config.height,
				};

				if renderer
					.render(
						&self.scene,
						&device_handle.device,
						&device_handle.queue,
						&mut encoder,
						&render_size,
						&texture_view,
					)
					.is_ok()
				{
					device_handle.queue.submit([encoder.finish()]);
					surface_texture.present();
				}

				// window.request_redraw();
			}
			_ => {}
		}
	}

	fn destroy_surfaces(&mut self, _event_loop: &dyn ActiveEventLoop) {
		self.render_state = RenderState::Suspended;
	}
}

fn main() -> Result<(), winit::error::EventLoopError> {
	tracing_subscriber::fmt::init();

	let app = ApplicationState::default();
	let event_loop = EventLoop::new()?;
	event_loop.run_app(app)
}
