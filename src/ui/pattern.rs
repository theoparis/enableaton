use gpui::{
    Context, FontWeight, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    SharedString, div, prelude::*, px, rgb,
};

use crate::daw::Daw;
use crate::models::DragKind;
use crate::theme::*;

impl Daw {
    pub(super) fn render_pattern_area(
        &mut self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let total_beats = self.total_bars * self.beats_per_bar;
        let total_w = BEAT_PX * total_beats as f32;
        let total_bars = self.total_bars;
        let bpb = self.beats_per_bar;
        let play_pos = self.play_pos;
        let n_tracks = self.tracks.len();
        let ph_x = play_pos * BEAT_PX;

        // ── Ruler ──────────────────────────────────────────────────────────────

        let mut ruler = div()
            .id("ruler")
            .relative()
            .flex_none()
            .h(px(RULER_H))
            .w(px(total_w))
            .bg(rgb(SURFACE))
            .border_b_1()
            .border_color(rgb(BORDER))
            .cursor_col_resize()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, ev: &MouseDownEvent, _w, cx| {
                    this.scrubbing = true;
                    this.scrub_to(f32::from(ev.position.x));
                    cx.notify();
                }),
            );

        for bar in 0..=total_bars {
            let x = bar as f32 * bpb as f32 * BEAT_PX;
            ruler = ruler
                .child(
                    div()
                        .absolute()
                        .left(px(x))
                        .top_0()
                        .w_px()
                        .h_full()
                        .bg(rgb(BORDER)),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(x + 2.))
                        .top(px(4.))
                        .text_xs()
                        .text_color(rgb(TEXT_DIM))
                        .child(SharedString::from(format!("{}", bar + 1))),
                );
        }
        ruler = ruler.child(
            div()
                .absolute()
                .left(px(ph_x))
                .top_0()
                .w(px(2.))
                .h_full()
                .bg(rgb(PLAYHEAD))
                .opacity(0.9),
        );

        let mut pattern_col = div().flex().flex_col().w(px(total_w)).child(ruler);

        // ── Track rows ─────────────────────────────────────────────────────────

        for t_idx in 0..n_tracks {
            let track = &self.tracks[t_idx];
            let color = track.color;
            let muted = track.muted;
            let track_name = track.name.clone();

            let mut row = div()
                .relative()
                .flex_none()
                .h(px(TRACK_H))
                .w(px(total_w))
                .bg(rgb(if muted { 0x111620 } else { BG }))
                .border_b_1()
                .border_color(rgb(BORDER));

            // Bar grid lines
            for bar in 0..=total_bars {
                let x = bar as f32 * bpb as f32 * BEAT_PX;
                row = row.child(
                    div()
                        .absolute()
                        .left(px(x))
                        .top_0()
                        .w_px()
                        .h_full()
                        .bg(rgb(BORDER))
                        .opacity(0.5),
                );
            }

            // Pattern blocks
            let active_p_idx = Daw::active_pattern_idx(track, play_pos);
            for p_idx in 0..track.patterns.len() {
                let pat = &track.patterns[p_idx];
                let x = pat.start as f32 * BEAT_PX;
                let w = (pat.len as f32 * BEAT_PX - 1.0).max(4.0);
                let blk_color = if muted { 0x1a2233u32 } else { color };
                let resize_id = SharedString::from(format!("rsz-{t_idx}-{p_idx}"));
                let move_id = SharedString::from(format!("pat-{t_idx}-{p_idx}"));
                let pat_len = pat.len;
                let name_clone = track_name.clone();
                let is_active = active_p_idx == Some(p_idx);

                let block = div()
                    .absolute()
                    .left(px(x))
                    .top(px(3.))
                    .w(px(w))
                    .h(px(TRACK_H - 6.))
                    .bg(rgb(if is_active && !muted {
                        // slightly lighten the block when it's the playing clip
                        (blk_color & 0x00fefefe).saturating_add(0x00101820)
                    } else {
                        blk_color
                    }))
                    .rounded_sm()
                    .border_l_2()
                    .border_color(rgb(if is_active {
                        PLAYHEAD
                    } else if muted {
                        TEXT_DIM
                    } else {
                        ACCENT
                    }))
                    .overflow_hidden()
                    // Label: name + length, clipped to block bounds
                    .child(if w >= 44.0 {
                        div()
                            .absolute()
                            .left(px(5.))
                            .top(px(4.))
                            .right(px(8.)) // don't overlap the resize handle
                            .overflow_hidden()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
                            .flex_none()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xc8dff0))
                                    .opacity(0.85)
                                    .flex_shrink()
                                    .overflow_hidden()
                                    .child(SharedString::from(name_clone)),
                            )
                            .child(
                                div()
                                    .flex_none()
                                    .text_xs()
                                    .text_color(rgb(0xffffff))
                                    .opacity(0.45)
                                    .child(SharedString::from(format!("{}b", pat_len))),
                            )
                    } else {
                        div()
                    })
                    // Resize handle
                    .child(
                        div()
                            .id(resize_id)
                            .absolute()
                            .right_0()
                            .top_0()
                            .w(px(6.))
                            .h_full()
                            .cursor_col_resize()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, ev: &MouseDownEvent, _w, cx| {
                                    let pat = &this.tracks[t_idx].patterns[p_idx];
                                    this.drag = Some(crate::models::DragState {
                                        track_idx: t_idx,
                                        pat_idx: p_idx,
                                        kind: DragKind::Resize,
                                        orig_start: pat.start,
                                        orig_len: pat.len,
                                        mouse_start_x: f32::from(ev.position.x),
                                    });
                                    cx.notify();
                                }),
                            ),
                    );

                let block_with_move = div()
                    .id(move_id)
                    .absolute()
                    .left(px(x))
                    .top(px(3.))
                    .w(px(w))
                    .h(px(TRACK_H - 6.))
                    .cursor_grab()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, ev: &MouseDownEvent, _w, cx| {
                            let mx = f32::from(ev.position.x);
                            let scroll_x = f32::from(this.pattern_scroll.offset().x);
                            let pat = &this.tracks[t_idx].patterns[p_idx];
                            let rel = mx - HEADER_W + scroll_x - pat.start as f32 * BEAT_PX;
                            if rel < pat.len as f32 * BEAT_PX - 6.0 {
                                this.drag = Some(crate::models::DragState {
                                    track_idx: t_idx,
                                    pat_idx: p_idx,
                                    kind: DragKind::Move,
                                    orig_start: pat.start,
                                    orig_len: pat.len,
                                    mouse_start_x: mx,
                                });
                                cx.notify();
                            }
                        }),
                    )
                    .child(block);

                row = row.child(block_with_move);
            }

            // Playhead line
            row = row.child(
                div()
                    .absolute()
                    .left(px(ph_x))
                    .top_0()
                    .w(px(2.))
                    .h_full()
                    .bg(rgb(PLAYHEAD))
                    .opacity(0.7),
            );

            pattern_col = pattern_col.child(row);
        }

        let scroll_handle = self.pattern_scroll.clone();
        div()
            .id("pattern-scroll")
            .flex_1()
            .overflow_x_scroll()
            .track_scroll(&scroll_handle)
            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, _w, cx| {
                if ev.pressed_button != Some(MouseButton::Left) {
                    return;
                }
                let x = f32::from(ev.position.x);
                if this.scrubbing {
                    this.scrub_to(x);
                    cx.notify();
                } else if this.drag.is_some() {
                    this.apply_drag(x);
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _w, cx| {
                    if this.drag.is_some() || this.scrubbing {
                        this.drag = None;
                        this.scrubbing = false;
                        cx.notify();
                    }
                }),
            )
            .child(pattern_col)
    }
}
