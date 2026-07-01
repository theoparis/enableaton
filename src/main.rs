mod audio;
mod daw;
mod models;
mod theme;
mod ui;

use gpui::{Bounds, KeyBinding, WindowBounds, WindowOptions, actions, prelude::*, px, size};

pub use daw::Daw;

actions!(daw, [Quit, TogglePlay]);

fn main() {
    gpui_platform::application().run(|cx| {
        let bounds = Bounds::centered(None, size(px(1280.), px(780.)), cx);

        let entity_slot: std::rc::Rc<std::cell::RefCell<Option<gpui::Entity<Daw>>>> =
            std::rc::Rc::new(std::cell::RefCell::new(None));
        let slot_fill = entity_slot.clone();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| {
                let entity = cx.new(|cx| Daw::new(cx));
                *slot_fill.borrow_mut() = Some(entity.clone());
                entity
            },
        )
        .expect("failed to open window");

        let daw_entity = entity_slot.borrow().clone().unwrap();
        cx.on_action(move |_: &TogglePlay, cx| {
            daw_entity.update(cx, |daw, cx| {
                daw.toggle_play(cx);
                cx.notify();
            });
        });
        cx.bind_keys([KeyBinding::new("space", TogglePlay, None)]);

        cx.activate(true);
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
    });
}
