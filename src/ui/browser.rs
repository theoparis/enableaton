use gpui::{ClickEvent, Context, FontWeight, IntoElement, SharedString, div, prelude::*, px, rgb};

use super::core::panel::{fl_window, window_title};
use crate::daw::Daw;
use crate::theme::*;

/// Static library folders (not wired to the filesystem yet).
const FOLDERS: &[(&str, u32)] = &[
    ("Recent files", 0x8fc66a),
    ("Plugin database", 0xe0a35a),
    ("Plugin presets", 0xd27fb0),
    ("Channel presets", 0x7aa8de),
    ("Mixer presets", 0x7aa8de),
    ("Scores", 0xc9c37a),
    ("Backup", 0x8fc66a),
    ("Clipboard files", 0xb5bcc2),
    ("Envelopes", 0xb5bcc2),
    ("Impulses", 0xb5bcc2),
    ("Packs", 0xe0a35a),
    ("Projects", 0x7aa8de),
    ("Recorded", 0xd9675e),
    ("Rendered", 0xb5bcc2),
    ("Sliced beats", 0xb5bcc2),
    ("Soundfonts", 0xb5bcc2),
    ("User", 0x8fc66a),
];

const ROW_H: f32 = 17.0;

fn folder_icon(color: u32) -> impl IntoElement {
    div()
        .flex_none()
        .w(px(11.))
        .h(px(8.))
        .rounded(px(1.))
        .bg(rgb(color))
        .border_t_2()
        .border_color(rgb(lighten(color, 0.3)))
}

fn row(indent: f32) -> gpui::Div {
    div()
        .flex_none()
        .h(px(ROW_H))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(5.))
        .pl(px(6. + indent))
        .pr(px(4.))
        .text_size(px(11.))
        .text_color(rgb(TEXT))
}

impl Daw {
    pub(super) fn render_browser(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let mut list = div()
            .id("browser-list")
            .flex_1()
            .flex()
            .flex_col()
            .py(px(2.))
            .overflow_y_scroll()
            .bg(rgb(INSET));

        // "Current project" is live: generators map to tracks.
        list = list
            .child(
                row(0.)
                    .child(div().text_size(px(8.)).text_color(rgb(TEXT_DIM)).child("▼"))
                    .child(folder_icon(ACCENT))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(TEXT_BRIGHT))
                            .child("Current project"),
                    ),
            )
            .child(
                row(12.)
                    .child(div().text_size(px(8.)).text_color(rgb(TEXT_DIM)).child("▼"))
                    .child(folder_icon(0xb5bcc2))
                    .child("Generators"),
            );

        for (idx, track) in self.tracks.iter().enumerate() {
            let selected = self.selected_track == Some(idx);
            list = list.child(
                row(30.)
                    .id(SharedString::from(format!("browser-gen-{idx}")))
                    .cursor_pointer()
                    .when(selected, |d| d.bg(rgb(0x3f5566)))
                    .hover(|s| s.bg(rgb(0x34424c)))
                    .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                        this.selected_track = Some(idx);
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex_none()
                            .w(px(7.))
                            .h(px(7.))
                            .rounded_full()
                            .bg(rgb(track.color)),
                    )
                    .child(
                        div()
                            .overflow_hidden()
                            .text_color(rgb(if selected { TEXT_BRIGHT } else { TEXT }))
                            .child(SharedString::from(track.name.clone())),
                    ),
            );
        }

        let n_patterns: usize = self.tracks.iter().map(|t| t.patterns.len()).sum();
        list = list.child(
            row(12.)
                .child(div().text_size(px(8.)).text_color(rgb(TEXT_DIM)).child("▶"))
                .child(folder_icon(0xb5bcc2))
                .child("Patterns")
                .child(
                    div()
                        .text_size(px(9.))
                        .text_color(rgb(TEXT_DIM))
                        .child(SharedString::from(format!("{n_patterns}"))),
                ),
        );

        for (i, (name, color)) in FOLDERS.iter().enumerate() {
            list = list.child(
                row(0.)
                    .id(SharedString::from(format!("browser-folder-{i}")))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x34424c)))
                    .child(div().text_size(px(8.)).text_color(rgb(TEXT_DIM)).child("▶"))
                    .child(folder_icon(*color))
                    .child(*name),
            );
        }

        fl_window()
            .flex_none()
            .w(px(BROWSER_W))
            .h_full()
            .child(
                window_title("☰", "Browser").child(
                    div()
                        .text_size(px(10.))
                        .text_color(rgb(TEXT_DIM))
                        .child("- All"),
                ),
            )
            .child(list)
    }
}
