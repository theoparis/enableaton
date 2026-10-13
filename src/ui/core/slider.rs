use std::sync::Arc;

use gpui::{
    App, AppContext as _, DragMoveEvent, Empty, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _, Window, div, px,
    relative, rgb,
};

use crate::theme::{ACCENT_DIM, BORDER, LCD_BG};

struct SliderDrag;

pub struct Slider {
    value: f32, // 0.0..=1.0
    on_change: Option<Arc<dyn Fn(f32, &mut App) + Send + Sync + 'static>>,
}

impl Slider {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            on_change: None,
        }
    }

    pub fn on_change(mut self, f: impl Fn(f32, &mut App) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

impl Render for Slider {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let slider_entity: Entity<Slider> = cx.entity();

        div()
            .id(("slider", slider_entity.entity_id()))
            .w_full()
            .h(px(20.))
            .flex()
            .items_center()
            .cursor_pointer()
            .on_drag(SliderDrag, |_, _, _, cx| cx.new(|_| Empty))
            .on_drag_move(
                move |ev: &DragMoveEvent<SliderDrag>, _window: &mut Window, app: &mut App| {
                    let x = ev.event.position.x;
                    let bounds = ev.bounds;

                    let width = f32::from(bounds.size.width);
                    if width <= 0.0 {
                        return;
                    }
                    let local_x = f32::from(x - bounds.left());
                    let new_val = (local_x / width).clamp(0.0, 1.0);

                    let cb = app.update_entity(&slider_entity, |slider, cx| {
                        slider.value = new_val;
                        cx.notify();
                        slider.on_change.clone()
                    });

                    if let Some(cb) = cb {
                        cb(new_val, app);
                    }
                },
            )
            .child(
                div()
                    .relative()
                    .w_full()
                    .h(px(4.))
                    .bg(rgb(LCD_BG))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .rounded_full()
                    .child(
                        div()
                            .w(relative(self.value))
                            .h_full()
                            .bg(rgb(ACCENT_DIM))
                            .rounded_full(),
                    )
                    // Handle
                    .child(
                        div()
                            .absolute()
                            .left(relative(self.value))
                            .top(px(-5.))
                            .ml(px(-4.))
                            .w(px(8.))
                            .h(px(12.))
                            .rounded(px(2.))
                            .bg(super::button::vgrad(0x9aa4ab, 0x5a646b))
                            .border_1()
                            .border_color(rgb(BORDER)),
                    ),
            )
    }
}
