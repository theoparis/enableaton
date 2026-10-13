use gpui::{
    ClickEvent, Context, FontWeight, IntoElement, MouseButton, MouseDownEvent, SharedString,
    deferred, div, prelude::*, px, rgb,
};

use super::core::button::{tool_btn, vgrad};
use super::core::panel::lcd;
use super::core::text::dim;
use crate::daw::Daw;
use crate::theme::*;

const MENU_NAMES: &[&str] = &[
    "FILE", "EDIT", "ADD", "PATTERNS", "VIEW", "OPTIONS", "TOOLS", "HELP",
];
const SCOPE_BARS: usize = 36;

type Action = fn(&mut Daw, &mut Context<Daw>);

struct MenuEntry {
    label: String,
    action: Action,
}

fn entry(label: impl Into<String>, action: Action) -> MenuEntry {
    MenuEntry {
        label: label.into(),
        action,
    }
}

fn checked(on: bool, label: &str) -> String {
    format!("{} {label}", if on { "✓" } else { "\u{2003}" })
}

fn menu_items(d: &Daw, menu: usize) -> Vec<MenuEntry> {
    match menu {
        0 => vec![entry("Quit", |_, cx| cx.quit())],
        1 => vec![
            entry("Clear selected pattern", |d, _| {
                if let Some(t) = d.selected_track {
                    let track = &mut d.tracks[t];
                    if let Some(p) = track.patterns.get_mut(track.active_pattern) {
                        p.notes.clear();
                        p.lanes.iter_mut().for_each(|l| *l = [false; STEPS]);
                        d.status = Some(format!("Cleared pattern {}", p.name));
                    }
                }
            }),
            entry("Remove clips of selected channel", |d, _| {
                if let Some(t) = d.selected_track {
                    d.tracks[t].clips.clear();
                    d.status = Some(format!("Removed clips of {}", d.tracks[t].name));
                }
            }),
        ],
        2 => vec![entry("Channel (synth)", |d, _| {
            d.add_track();
            d.selected_track = Some(d.tracks.len() - 1);
            d.status = Some("Added channel".into());
        })],
        3 => vec![
            entry("New pattern", |d, _| {
                if let Some(t) = d.selected_track {
                    let idx = d.tracks[t].add_pattern();
                    d.tracks[t].active_pattern = idx;
                    d.status = Some("New pattern".into());
                }
            }),
            entry("Clone pattern", |d, _| {
                if let Some(t) = d.selected_track {
                    let track = &mut d.tracks[t];
                    if let Some(src) = track.patterns.get(track.active_pattern).cloned() {
                        let mut copy = src;
                        copy.name = format!("P{}", track.patterns.len() + 1);
                        track.patterns.push(copy);
                        track.active_pattern = track.patterns.len() - 1;
                        d.status = Some("Cloned pattern".into());
                    }
                }
            }),
        ],
        4 => vec![
            entry(checked(d.show_browser, "Browser"), |d, _| {
                d.show_browser ^= true
            }),
            entry(checked(d.show_channel_rack, "Channel rack"), |d, _| {
                d.show_channel_rack ^= true
            }),
            entry(checked(d.show_piano_roll, "Piano roll"), |d, _| {
                d.show_piano_roll ^= true
            }),
            entry(checked(d.show_playlist, "Playlist"), |d, _| {
                d.show_playlist ^= true
            }),
            entry(checked(d.show_mixer, "Mixer"), |d, _| d.show_mixer ^= true),
        ],
        5 => vec![
            entry("Reset layout", |d, _| {
                d.browser_w = BROWSER_W;
                d.mid_w = MID_COL_W;
                d.rack_h = RACK_H;
                d.mixer_h = MIXER_H;
                d.show_browser = true;
                d.show_channel_rack = true;
                d.show_piano_roll = true;
                d.show_playlist = true;
                d.show_mixer = true;
                d.status = Some("Layout reset".into());
            }),
            entry("Piano roll zoom in", |d, _| {
                d.piano_roll_zoom = (d.piano_roll_zoom + 0.5).min(8.0)
            }),
            entry("Piano roll zoom out", |d, _| {
                d.piano_roll_zoom = (d.piano_roll_zoom - 0.5).max(0.5)
            }),
        ],
        6 => vec![
            entry("Mute all channels", |d, _| {
                d.tracks.iter_mut().for_each(|t| t.muted = true)
            }),
            entry("Unmute all channels", |d, _| {
                d.tracks.iter_mut().for_each(|t| t.muted = false)
            }),
        ],
        _ => vec![entry("About Enableaton", |d, _| {
            d.status = Some(format!("Enableaton {}", env!("CARGO_PKG_VERSION")));
        })],
    }
}

fn separator() -> impl IntoElement {
    div()
        .flex_none()
        .w(px(1.))
        .h(px(30.))
        .mx(px(4.))
        .bg(rgb(BORDER))
}

impl Daw {
    pub(super) fn render_toolbar(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let bpm = self.bpm;
        let playing = self.playing;
        let recording = self.recording;
        let song_mode = self.song_mode;
        let bpb = self.beats_per_bar as f32;
        let pos = self.play_pos;
        let bar = (pos / bpb).floor() as usize + 1;
        let step = ((pos % bpb) * 4.0).floor() as usize + 1;
        let tick = ((pos * 4.0).fract() * 24.0).floor() as usize;

        let pattern_label = match self.selected_track {
            Some(t) => {
                let track = &self.tracks[t];
                let pat = track
                    .patterns
                    .get(track.active_pattern)
                    .map_or("—", |p| p.name.as_str());
                format!("{} · {pat}", track.name)
            }
            None => "(none)".to_string(),
        };

        // ── Menu bar with dropdowns ────────────────────────────────────────────
        let mut menu_bar = div().flex().flex_row().gap(px(2.));
        for (m_idx, name) in MENU_NAMES.iter().enumerate() {
            let open = self.open_menu == Some(m_idx);
            let mut item = div()
                .id(SharedString::from(format!("menu-{m_idx}")))
                .relative()
                .px(px(4.))
                .text_size(px(10.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(if open { ACCENT } else { TEXT }))
                .cursor_pointer()
                .hover(|s| s.text_color(rgb(ACCENT)))
                .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                    this.open_menu = if this.open_menu == Some(m_idx) {
                        None
                    } else {
                        Some(m_idx)
                    };
                    cx.notify();
                }))
                .child(*name);
            if open {
                let mut list = div()
                    .absolute()
                    .top(px(14.))
                    .left(px(0.))
                    .w(px(170.))
                    .py(px(3.))
                    .flex()
                    .flex_col()
                    .rounded(px(3.))
                    .bg(rgb(0x2b3237))
                    .border_1()
                    .border_color(rgb(BEVEL))
                    .shadow_md();
                for (i_idx, entry) in menu_items(self, m_idx).into_iter().enumerate() {
                    let action = entry.action;
                    list = list.child(
                        div()
                            .id(SharedString::from(format!("menu-{m_idx}-{i_idx}")))
                            .px(px(10.))
                            .h(px(18.))
                            .flex()
                            .items_center()
                            .text_size(px(11.))
                            .text_color(rgb(TEXT))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(ACCENT_DIM)).text_color(rgb(TEXT_BRIGHT)))
                            .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                                cx.stop_propagation();
                                this.open_menu = None;
                                action(this, cx);
                                cx.notify();
                            }))
                            .child(SharedString::from(entry.label)),
                    );
                }
                item = item.child(deferred(list).priority(2));
            }
            menu_bar = menu_bar.child(item);
        }

        let menu = div()
            .flex_none()
            .w(px(300.))
            .flex()
            .flex_col()
            .justify_center()
            .gap(px(3.))
            .child(menu_bar)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.))
                    .child(
                        div()
                            .flex_1()
                            .px(px(4.))
                            .rounded(px(2.))
                            .bg(rgb(INSET))
                            .text_size(px(10.))
                            .text_color(rgb(TEXT_DIM))
                            .overflow_hidden()
                            .child(SharedString::from(self.status.clone().unwrap_or_else(
                                || format!("untitled.enb — {} channels", self.tracks.len()),
                            ))),
                    )
                    .child(dim("VOL"))
                    .child(div().w(px(56.)).child(self.master_slider.clone())),
            );

        // ── Transport ──────────────────────────────────────────────────────────
        let mode_switch = div().flex_none().flex().flex_col().gap(px(2.)).children(
            [("PAT", !song_mode), ("SONG", song_mode)].map(|(label, on)| {
                div()
                    .id(SharedString::from(format!("mode-{label}")))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(3.))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _: &ClickEvent, _w, cx| {
                        this.song_mode = label == "SONG";
                        cx.notify();
                    }))
                    .child(
                        div()
                            .w(px(7.))
                            .h(px(7.))
                            .rounded_full()
                            .bg(rgb(if on { ACCENT } else { LED_OFF }))
                            .border_1()
                            .border_color(rgb(BORDER)),
                    )
                    .child(
                        div()
                            .text_size(px(9.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(if on { TEXT_BRIGHT } else { TEXT_DIM }))
                            .child(label),
                    )
            }),
        );

        let transport = div()
            .flex_none()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(3.))
            .child(tool_btn(
                "btn-play",
                if playing { "❚❚" } else { "▶" },
                26.,
                playing,
                LED_ON,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.toggle_play(cx);
                    cx.notify();
                }),
            ))
            .child(tool_btn(
                "btn-stop",
                "■",
                26.,
                false,
                ACCENT,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.playing = false;
                    this.play_pos = 0.0;
                    this.stop_tick();
                    cx.notify();
                }),
            ))
            .child(tool_btn(
                "btn-rec",
                "●",
                26.,
                recording,
                RED,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.recording = !this.recording;
                    cx.notify();
                }),
            ));

        // Tempo — drag up/down to change.
        let tempo = lcd()
            .id("bpm-drag")
            .h(px(26.))
            .flex_col()
            .justify_center()
            .items_end()
            .cursor_ns_resize()
            .hover(|s| s.border_color(rgb(ACCENT_DIM)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, ev: &MouseDownEvent, _w, _cx| {
                    this.bpm_drag_origin = Some((f32::from(ev.position.y), this.bpm));
                }),
            )
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::BOLD)
                    .child(SharedString::from(format!("{bpm}.000"))),
            )
            .child(
                div()
                    .text_size(px(7.))
                    .text_color(rgb(TEXT_DIM))
                    .mt(px(-3.))
                    .child("TEMPO"),
            );

        let time = lcd()
            .h(px(32.))
            .gap(px(4.))
            .child(
                div()
                    .text_size(px(20.))
                    .font_weight(FontWeight::BOLD)
                    .child(SharedString::from(format!("{bar}:{step:02}"))),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(7.))
                            .text_color(rgb(TEXT_DIM))
                            .child("B:S:T"),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(FontWeight::BOLD)
                            .child(SharedString::from(format!("{tick:02}"))),
                    ),
            );

        // Decorative oscilloscope that wiggles while playing.
        let scope = div()
            .flex_none()
            .w(px(96.))
            .h(px(30.))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(1.))
            .px(px(3.))
            .rounded(px(2.))
            .bg(rgb(LCD_BG))
            .border_1()
            .border_color(rgb(BORDER))
            .children((0..SCOPE_BARS).map(|i| {
                let amp = if playing {
                    let t = pos * 9.0 + i as f32 * 0.55;
                    (t.sin() * (t * 2.3).cos()).abs() * (1.0 - (pos * 4.0).fract() * 0.6)
                } else {
                    0.04
                };
                div()
                    .flex_1()
                    .h(px((amp * 24.0).max(1.0)))
                    .bg(rgb(0xb7c9d6))
            }));

        let pattern_sel = lcd()
            .id("pattern-sel")
            .cursor_pointer()
            .on_click(cx.listener(|this, _: &ClickEvent, _w, cx| {
                // Cycle the selected channel's active pattern.
                if let Some(t) = this.selected_track {
                    let track = &mut this.tracks[t];
                    if !track.patterns.is_empty() {
                        track.active_pattern = (track.active_pattern + 1) % track.patterns.len();
                    }
                }
                cx.notify();
            }))
            .h(px(22.))
            .w(px(150.))
            .gap(px(4.))
            .child(div().text_size(px(9.)).text_color(rgb(ACCENT)).child("▤"))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .text_size(px(10.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(SharedString::from(pattern_label)),
            )
            .child(div().text_size(px(8.)).text_color(rgb(TEXT_DIM)).child("▼"));

        let windows = div()
            .flex_none()
            .flex()
            .flex_row()
            .gap(px(2.))
            .child(tool_btn(
                "win-browser",
                "☰",
                22.,
                self.show_browser,
                ACCENT,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.show_browser ^= true;
                    cx.notify();
                }),
            ))
            .child(tool_btn(
                "win-rack",
                "▦",
                22.,
                self.show_channel_rack,
                ACCENT,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.show_channel_rack ^= true;
                    cx.notify();
                }),
            ))
            .child(tool_btn(
                "win-piano",
                "♪",
                22.,
                self.show_piano_roll,
                ACCENT,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.show_piano_roll ^= true;
                    cx.notify();
                }),
            ))
            .child(tool_btn(
                "win-playlist",
                "▤",
                22.,
                self.show_playlist,
                ACCENT,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.show_playlist ^= true;
                    cx.notify();
                }),
            ))
            .child(tool_btn(
                "win-mixer",
                "▥",
                22.,
                self.show_mixer,
                ACCENT,
                cx.listener(|this, _: &ClickEvent, _w, cx| {
                    this.show_mixer ^= true;
                    cx.notify();
                }),
            ));

        div()
            .flex_none()
            .h(px(TOOLBAR_H))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .px(px(8.))
            .bg(vgrad(0x4b555d, 0x3c454c))
            .border_b_1()
            .border_color(rgb(BORDER))
            .child(menu)
            .child(separator())
            .child(mode_switch)
            .child(transport)
            .child(tempo)
            .child(separator())
            .child(time)
            .child(scope)
            .child(separator())
            .child(pattern_sel)
            .child(windows)
            .child(div().flex_1())
            .child(
                div()
                    .flex_none()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_size(px(14.))
                            .text_color(rgb(ACCENT))
                            .font_weight(FontWeight::BOLD)
                            .child("◆"),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(TEXT))
                            .child(SharedString::from(format!(
                                "Enableaton {}",
                                env!("CARGO_PKG_VERSION")
                            ))),
                    ),
            )
    }
}
