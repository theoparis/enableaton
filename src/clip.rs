use std::sync::Arc;

use xilem::{
	Affine, Color, Pod, ViewCtx,
	core::{MessageCtx, MessageResult, Mut, View, ViewArgument, ViewMarker},
	masonry::{
		accesskit::Role,
		core::{ChildrenIds, Widget},
		kurbo::{BezPath, Point, Size, Stroke},
		peniko::Fill,
	},
	palette,
};

pub struct Clip {
	pub track: usize,
	pub start: f64,
	pub duration: f64,
	pub color: Color,
	pub data: Vec<f32>,
}

#[derive(Clone)]
pub struct ClipView {
	clip: Arc<Clip>,
	bpm: f64,
	sample_rate: u32,
}

impl ClipView {
	pub fn new(clip: Arc<Clip>, bpm: f64, sample_rate: u32) -> Self {
		Self {
			clip,
			bpm,
			sample_rate,
		}
	}
}

impl Widget for ClipView {
	type Action = ();

	fn register_children(
		&mut self,
		ctx: &mut xilem::masonry::core::RegisterCtx<'_>,
	) {
	}

	fn layout(
		&mut self,
		ctx: &mut xilem::masonry::core::LayoutCtx<'_>,
		props: &mut xilem::masonry::core::PropertiesMut<'_>,
		bc: &xilem::masonry::core::BoxConstraints,
	) -> xilem::masonry::kurbo::Size {
		let size = Size::new(bc.max().width, 100.0);
		bc.constrain(size)
	}

	fn paint(
		&mut self,
		ctx: &mut xilem::masonry::core::PaintCtx<'_>,
		_props: &xilem::masonry::core::PropertiesRef<'_>,
		scene: &mut xilem::masonry::vello::Scene,
	) {
		let size = ctx.size();
		let rect = size.to_rect();
		scene.fill(
			Fill::NonZero,
			Affine::IDENTITY,
			self.clip.color,
			None,
			&rect.to_rounded_rect(10.0),
		);

		let samples_per_point = self.clip.data.len() / 1024;

		let chunks: Vec<_> = self.clip.data.chunks(samples_per_point).collect();
		let total_chunks = chunks.len();

		let waveform_path = chunks.iter().enumerate().fold(
			BezPath::new(),
			|mut path, (i, chunk)| {
				let avg = chunk.iter().sum::<f32>() / chunk.len() as f32;
				let x = (i as f64 / total_chunks.max(1) as f64) * size.width;

				let y_center = size.height * 0.5;
				let y_amplitude = size.height * 0.4;
				let y = y_center + (avg as f64 * y_amplitude);
				let y = y.clamp(5.0, size.height - 5.0);

				let point = Point::new(x, y);

				if i == 0 {
					path.move_to(point);
				} else {
					path.line_to(point);
				}
				path
			},
		);

		scene.stroke(
			&Stroke::new(2.0),
			Affine::IDENTITY,
			Color::WHITE,
			None,
			&waveform_path,
		);
	}

	fn accessibility_role(&self) -> xilem::masonry::accesskit::Role {
		Role::Canvas
	}

	fn accessibility(
		&mut self,
		_ctx: &mut xilem::masonry::core::AccessCtx<'_>,
		_props: &xilem::masonry::core::PropertiesRef<'_>,
		_node: &mut xilem::masonry::accesskit::Node,
	) {
	}

	fn children_ids(&self) -> xilem::masonry::core::ChildrenIds {
		ChildrenIds::new()
	}
}

impl ViewMarker for ClipView {}
impl<State: ViewArgument, Action> View<State, Action, ViewCtx> for ClipView {
	type Element = Pod<ClipView>;
	type ViewState = ();

	fn build(
		&self,
		ctx: &mut ViewCtx,
		_: State::Params<'_>,
	) -> (Self::Element, Self::ViewState) {
		ctx.with_leaf_action_widget(|ctx| ctx.create_pod(self.clone()))
	}

	fn rebuild(
		&self,
		_prev: &Self,
		(): &mut Self::ViewState,
		_ctx: &mut ViewCtx,
		mut _element: Mut<'_, Self::Element>,
		_: State::Params<'_>,
	) {
	}

	fn teardown(
		&self,
		(): &mut Self::ViewState,
		ctx: &mut ViewCtx,
		element: Mut<'_, Self::Element>,
	) {
		ctx.teardown_leaf(element);
	}

	fn message(
		&self,
		(): &mut Self::ViewState,
		_message: &mut MessageCtx,
		_element: Mut<'_, Self::Element>,
		app_state: State::Params<'_>,
	) -> MessageResult<Action> {
		MessageResult::Nop
	}
}
