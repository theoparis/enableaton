use gpui::{ClickEvent, Context, FontWeight, IntoElement, SharedString, div, prelude::*, px, rgb};

use super::{dim, sample_badge};
use crate::daw::Daw;
use crate::theme::*;

impl Daw {
    pub(super) fn render_headers(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let n_tracks = self.tracks.len();
        let selected = self.selected_track;

        let mut headers = div()
            .flex_none()
            .w(px(HEADER_W))
            .flex()
            .flex_col()
            .bg(rgb(SURFACE))
            .border_r_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .h(px(RULER_H))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .px_2()
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            .child("▤"),
                    )
                    .child(dim("SONG EDITOR")),
            );

        for idx in 0..n_tracks {
            let t = &self.tracks[idx];
            let color = t.color;
            let muted = t.muted;
            let name = SharedString::from(t.name.clone());
            let badge_label = t.sample.short_label();
            let badge_color = t.sample.badge_color();
            let is_selected = selected == Some(idx);

            headers = headers.child(
                div()
                    .id(SharedString::from(format!("track-row-{idx}")))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .px_2()
                    .h(px(TRACK_H))
                    .bg(rgb(if is_selected { 0x1e3050 } else { SURFACE }))
                    .border_b_1()
                    .border_color(rgb(if is_selected { ACCENT_DIM } else { BORDER }))
                    // Left accent stripe — thicker when selected
                    .child(
                        div()
                            .w(px(if is_selected { 4. } else { 3. }))
                            .h_6()
                            .rounded_full()
                            .bg(rgb(if is_selected { ACCENT } else { color })),
                    )
                    // Sample type badge
                    .child(sample_badge(badge_label, badge_color))
                    // Track name — click to preview sound
                    .child(
                        div()
                            .id(SharedString::from(format!("preview-{idx}")))
                            .flex_1()
                            .min_w(px(0.))
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(if muted {
                                TEXT_DIM
                            } else if is_selected {
                                0xddf0ff
                            } else {
                                TEXT
                            }))
                            .overflow_hidden()
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                                this.selected_track = Some(idx);
                                if let Some(audio) = &this.audio {
                                    audio.trigger(this.tracks[idx].sample);
                                }
                                cx.notify();
                            }))
                            .child(name),
                    )
                    // Mute button
                    .child(
                        div()
                            .id(SharedString::from(format!("mute-{idx}")))
                            .flex_none()
                            .w(px(18.))
                            .h(px(18.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(if muted { ACCENT } else { TEXT_DIM }))
                            .bg(rgb(if muted { ACCENT_DIM } else { SURFACE2 }))
                            .cursor_pointer()
                            .hover(|s| s.text_color(rgb(ACCENT)))
                            .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                                this.tracks[idx].muted ^= true;
                                cx.notify();
                            }))
                            .child("M"),
                    ),
            );
        }

        headers.child(
            div()
                .id("add-track")
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap_1()
                .h_8()
                .mx_2()
                .my_1()
                .rounded_sm()
                .bg(rgb(SURFACE2))
                .border_1()
                .border_color(rgb(BORDER))
                .text_xs()
                .text_color(rgb(TEXT_DIM))
                .cursor_pointer()
                .hover(|s| s.bg(rgb(BORDER)).text_color(rgb(TEXT)))
                .on_click(cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.add_track();
                    cx.notify();
                }))
                .child("＋")
                .child("Add Track"),
        )
    }
}
