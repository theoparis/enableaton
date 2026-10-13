//! Pane entities. Each heavy pane is its own view so gpui can reuse its
//! prepaint/paint when nothing it draws changed. The playhead, which moves
//! every frame, is drawn by overlay canvases outside the cached panes, so a
//! playback tick never dirties the pane underneath it.

use gpui::{
    AnyElement, App, AppContext, Bounds, ContentMask, Context, Entity, IntoElement, Pixels, Render,
    Styled, Subscription, Window, canvas, div, point, px, rgb, size,
};

use crate::daw::Daw;
use crate::models::Instrument;
use crate::theme::*;

/// Pure notification source: observers re-render whenever it is notified.
pub struct Ticker;

#[derive(Clone, Copy)]
pub enum Pane {
    Toolbar,
    Browser,
    Rack,
    Lower,
    Playlist,
    Mixer,
}

/// Renders one `Daw` pane. Re-renders when `Daw` is notified (coarse: user
/// edits, step boundaries) or when its optional extra ticker fires.
pub struct PaneView {
    daw: Entity<Daw>,
    pane: Pane,
    _subs: Vec<Subscription>,
}

impl PaneView {
    fn new(
        daw: &Entity<Daw>,
        pane: Pane,
        ticker: Option<&Entity<Ticker>>,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut subs = vec![cx.observe(daw, |_, _, cx| cx.notify())];
        if let Some(t) = ticker {
            subs.push(cx.observe(t, |_, _, cx| cx.notify()));
        }
        Self {
            daw: daw.clone(),
            pane,
            _subs: subs,
        }
    }
}

impl Render for PaneView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane = self.pane;
        self.daw.update(cx, |d, cx| -> AnyElement {
            match pane {
                Pane::Toolbar => d.render_toolbar(cx).into_any_element(),
                Pane::Browser => d.render_browser(cx).into_any_element(),
                Pane::Rack => d.render_channel_rack(cx).into_any_element(),
                Pane::Playlist => d.render_playlist(cx).into_any_element(),
                Pane::Mixer => d.render_mixer(cx).into_any_element(),
                Pane::Lower => match d.selected_track.map(|i| &d.tracks[i].instrument) {
                    Some(Instrument::DrumRack(_)) => d.render_drum_settings(cx).into_any_element(),
                    Some(Instrument::Plugin(_)) if d.show_device => {
                        d.render_device_panel(cx).into_any_element()
                    }
                    _ => d.render_piano_roll(cx).into_any_element(),
                },
            }
        })
    }
}

/// Paints a 1-px vertical line at `x` from `top` to `bottom`, clipped to `clip`.
fn playhead_line(
    window: &mut Window,
    clip: Bounds<Pixels>,
    x: Pixels,
    top: Pixels,
    bottom: Pixels,
) {
    window.with_content_mask(
        Some(ContentMask {
            bounds: clip,
            ..Default::default()
        }),
        |window| {
            window.paint_quad(gpui::fill(
                Bounds::new(point(x, top), size(px(1.), bottom - top)),
                rgb(PLAYHEAD),
            ));
        },
    );
}

/// Playlist playhead: ruler marker plus a line across the track rows.
pub struct PlaylistOverlay {
    daw: Entity<Daw>,
    _subs: Vec<Subscription>,
}

impl Render for PlaylistOverlay {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let d = self.daw.read(cx);
        let ph_x = d.play_pos * BEAT_PX;
        let scroll = d.pattern_scroll.clone();
        let grid = d.playlist_grid.clone();
        let grid_h = d.tracks.len().max(MIN_PLAYLIST_ROWS) as f32 * TRACK_H;
        canvas(
            |_, _, _| {},
            move |_, _, window: &mut Window, _: &mut App| {
                let b = grid.get();
                if b.size.width <= px(0.) {
                    return;
                }
                let x = b.left() + scroll.offset().x + px(ph_x);
                let rows_top = b.top() + px(RULER_H);
                let rows_bottom = (rows_top + px(grid_h)).min(b.bottom());
                playhead_line(window, b, x, rows_top, rows_bottom);
                // Ruler marker: small downward triangle ending at the ruler's bottom edge.
                window.with_content_mask(
                    Some(ContentMask {
                        bounds: b,
                        ..Default::default()
                    }),
                    |window| {
                        let mut pb = gpui::PathBuilder::fill();
                        pb.move_to(point(x - px(5.), rows_top - px(7.)));
                        pb.line_to(point(x + px(6.), rows_top - px(7.)));
                        pb.line_to(point(x + px(0.5), rows_top));
                        pb.close();
                        if let Ok(path) = pb.build() {
                            window.paint_path(path, rgb(PLAYHEAD));
                        }
                    },
                );
            },
        )
        .absolute()
        .size_full()
    }
}

/// Piano-roll playhead: shown while a clip of the edited pattern is under
/// the timeline playhead.
pub struct PianoRollOverlay {
    daw: Entity<Daw>,
    _subs: Vec<Subscription>,
}

impl Render for PianoRollOverlay {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let d = self.daw.read(cx);
        let Some(t_idx) = d.selected_track else {
            return div().into_any_element();
        };
        let track = &d.tracks[t_idx];
        let shows_roll = match track.instrument {
            Instrument::DrumRack(_) => false,
            Instrument::Plugin(_) => !d.show_device,
            _ => true,
        };
        let Some(pat) = track.patterns.get(track.active_pattern) else {
            return div().into_any_element();
        };
        if !shows_roll {
            return div().into_any_element();
        }
        let pat_beats = pat.len as f32;
        let p_idx = track.active_pattern;
        let pos = d.play_pos;
        let local_ph = track.clips.iter().find_map(|c| {
            let cs = c.start as f32;
            (c.pattern == p_idx && pos >= cs && pos < cs + pat_beats).then_some(pos - cs)
        });
        let Some(lp) = local_ph else {
            return div().into_any_element();
        };
        let beat_px = BEAT_PX * d.piano_roll_zoom;
        let scroll = d.piano_roll_scroll.clone();
        let grid = d.piano_roll_grid.clone();
        canvas(
            |_, _, _| {},
            move |_, _, window: &mut Window, _: &mut App| {
                let b = grid.get();
                if b.size.width <= px(0.) {
                    return;
                }
                let x = b.left() + scroll.offset().x + px(lp * beat_px);
                playhead_line(window, b, x, b.top(), b.bottom());
            },
        )
        .absolute()
        .size_full()
        .into_any_element()
    }
}

/// All pane entities, created once from `Daw::render`.
#[derive(Clone)]
pub struct Panes {
    pub toolbar: Entity<PaneView>,
    pub browser: Entity<PaneView>,
    pub rack: Entity<PaneView>,
    pub lower: Entity<PaneView>,
    pub playlist: Entity<PaneView>,
    pub mixer: Entity<PaneView>,
    pub playlist_overlay: Entity<PlaylistOverlay>,
    pub piano_roll_overlay: Entity<PianoRollOverlay>,
}

impl Panes {
    pub fn new(
        daw: &Entity<Daw>,
        ticker: Entity<Ticker>,
        meter_clock: Entity<Ticker>,
        cx: &mut Context<Daw>,
    ) -> Self {
        let pane = |p: Pane, t: Option<&Entity<Ticker>>, cx: &mut Context<Daw>| {
            cx.new(|cx| PaneView::new(daw, p, t, cx))
        };
        Self {
            toolbar: pane(Pane::Toolbar, Some(&ticker), cx),
            browser: pane(Pane::Browser, None, cx),
            rack: pane(Pane::Rack, None, cx),
            lower: pane(Pane::Lower, None, cx),
            playlist: pane(Pane::Playlist, None, cx),
            mixer: pane(Pane::Mixer, Some(&meter_clock), cx),
            playlist_overlay: cx.new(|cx| PlaylistOverlay {
                daw: daw.clone(),
                _subs: overlay_subs(daw, &ticker, cx),
            }),
            piano_roll_overlay: cx.new(|cx| PianoRollOverlay {
                daw: daw.clone(),
                _subs: overlay_subs(daw, &ticker, cx),
            }),
        }
    }
}

/// Overlays re-render on every `Daw` notify and on every tick.
fn overlay_subs<T: 'static>(
    daw: &Entity<Daw>,
    ticker: &Entity<Ticker>,
    cx: &mut Context<T>,
) -> Vec<Subscription> {
    vec![
        cx.observe(daw, |_, _, cx| cx.notify()),
        cx.observe(ticker, |_, _, cx| cx.notify()),
    ]
}
