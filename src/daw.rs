use gpui::{AppContext, Context, ScrollHandle, Task, point, px};
use std::time::{Duration, Instant};

use crate::audio::{AudioEngine, SampleType};
use crate::models::{DragKind, DragState, Pattern, Track};
use crate::theme::*;

pub struct Daw {
    pub playing: bool,
    pub recording: bool,
    pub bpm: u32,
    pub play_pos: f32,
    pub last_step: i32,
    pub tracks: Vec<Track>,
    pub beats_per_bar: usize,
    pub total_bars: usize,
    pub drag: Option<DragState>,
    pub scrubbing: bool,
    pub show_beat_editor: bool,
    pub audio: Option<AudioEngine>,
    pub _tick_task: Option<Task<()>>,
    /// Currently selected track index (for editing).
    pub selected_track: Option<usize>,
    /// (mouse_start_y, bpm_at_drag_start) for the BPM drag slider.
    pub bpm_drag_origin: Option<(f32, u32)>,
    /// Scroll handle for the pattern editor — used to auto-follow the playhead.
    pub pattern_scroll: ScrollHandle,
}

impl Daw {
    pub fn new() -> Self {
        let audio = AudioEngine::start();
        if audio.is_none() {
            eprintln!("warning: no audio device found, sequencer will be silent");
        }

        // Kick: four-on-the-floor
        let kick_steps = [
            true, false, false, false, true, false, false, false, true, false, false, false, true,
            false, false, false,
        ];
        // Snare: 2 and 4
        let snare_steps = [
            false, false, false, false, true, false, false, false, false, false, false, false,
            true, false, false, false,
        ];
        // Hi-hat: 8th notes
        let hihat_steps = [
            true, false, true, false, true, false, true, false, true, false, true, false, true,
            false, true, false,
        ];
        // Open hat
        let openhat_steps = [
            false, false, false, false, false, false, true, false, false, false, false, false,
            false, false, true, false,
        ];
        // Clap
        let clap_steps = [
            false, false, false, false, true, false, false, true, false, false, false, false, true,
            false, false, false,
        ];
        // Bass
        let bass_steps = [
            true, false, false, true, false, false, true, false, true, false, false, false, true,
            false, false, true,
        ];
        // Synth lead
        let synth_steps = [
            true, false, false, false, false, false, true, false, false, true, false, false, true,
            false, false, false,
        ];
        // Pad: sparse
        let pad_steps = [
            true, false, false, false, false, false, false, false, true, false, false, false,
            false, false, false, false,
        ];

        fn pats(ranges: &[(usize, usize)], steps: [bool; STEPS]) -> Vec<Pattern> {
            ranges
                .iter()
                .map(|&(start, len)| Pattern { start, len, steps })
                .collect()
        }

        Self {
            playing: false,
            recording: false,
            bpm: 128,
            play_pos: 0.0,
            last_step: -1,
            beats_per_bar: 4,
            total_bars: 32,
            drag: None,
            scrubbing: false,
            show_beat_editor: true,
            audio,
            _tick_task: None,
            selected_track: None,
            bpm_drag_origin: None,
            pattern_scroll: ScrollHandle::default(),
            tracks: vec![
                Track {
                    name: "Kick".into(),
                    color: TRACK_COLORS[0],
                    muted: false,
                    sample: SampleType::Kick,
                    patterns: pats(&[(0, 4), (4, 4), (12, 8), (24, 8)], kick_steps),
                },
                Track {
                    name: "Snare".into(),
                    color: TRACK_COLORS[1],
                    muted: false,
                    sample: SampleType::Snare,
                    patterns: pats(&[(0, 8), (16, 8)], snare_steps),
                },
                Track {
                    name: "Hi-Hat".into(),
                    color: TRACK_COLORS[2],
                    muted: false,
                    sample: SampleType::HiHat,
                    patterns: pats(&[(0, 16), (20, 4)], hihat_steps),
                },
                Track {
                    name: "Open Hat".into(),
                    color: TRACK_COLORS[3],
                    muted: false,
                    sample: SampleType::OpenHat,
                    patterns: pats(&[(4, 4), (20, 4)], openhat_steps),
                },
                Track {
                    name: "Clap".into(),
                    color: TRACK_COLORS[4],
                    muted: false,
                    sample: SampleType::Clap,
                    patterns: pats(&[(2, 6), (18, 10)], clap_steps),
                },
                Track {
                    name: "Bass".into(),
                    color: TRACK_COLORS[5],
                    muted: false,
                    sample: SampleType::Bass,
                    patterns: pats(&[(2, 6), (10, 4), (18, 10)], bass_steps),
                },
                Track {
                    name: "Synth Lead".into(),
                    color: TRACK_COLORS[6],
                    muted: false,
                    sample: SampleType::SynthLead,
                    patterns: pats(&[(4, 8), (16, 12)], synth_steps),
                },
                Track {
                    name: "Pad".into(),
                    color: TRACK_COLORS[0],
                    muted: false,
                    sample: SampleType::Pad,
                    patterns: pats(&[(0, 28)], pad_steps),
                },
            ],
        }
    }

    pub fn start_tick(&mut self, cx: &mut Context<Self>) {
        let mut last_tick = Instant::now();
        let task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_spawn(async {
                    std::thread::sleep(Duration::from_millis(8));
                })
                .await;

                let keep_going = this
                    .update(cx, |this, cx| {
                        if !this.playing {
                            return false;
                        }
                        let now = Instant::now();
                        let elapsed_secs = (now - last_tick).as_secs_f32();
                        last_tick = now;
                        let beats_per_tick = this.bpm as f32 / 60.0 * elapsed_secs;
                        this.play_pos += beats_per_tick;
                        let total = (this.total_bars * this.beats_per_bar) as f32;
                        if this.play_pos >= total {
                            this.play_pos = 0.0;
                        }

                        let steps_per_beat = 4_usize;
                        let global_step = (this.play_pos * steps_per_beat as f32) as i32;
                        if global_step != this.last_step {
                            this.last_step = global_step;
                            for t in &this.tracks {
                                if t.muted {
                                    continue;
                                }
                                // Find which pattern the playhead is inside.
                                if let Some(pat) = t.patterns.iter().find(|pat| {
                                    let start = pat.start as f32;
                                    let end = (pat.start + pat.len) as f32;
                                    this.play_pos >= start && this.play_pos < end
                                }) {
                                    // Which 16th-note step within this specific pattern?
                                    let elapsed = this.play_pos - pat.start as f32;
                                    let step =
                                        (elapsed * steps_per_beat as f32).floor() as usize % STEPS;
                                    if pat.steps[step] {
                                        if let Some(audio) = &this.audio {
                                            audio.trigger(t.sample);
                                        }
                                    }
                                }
                            }
                        }

                        // Auto-scroll the song editor to keep the playhead visible.
                        let ph_x = this.play_pos * BEAT_PX;
                        // Scroll so the playhead sits ~150px in from the left.
                        let target_offset = (ph_x - 150.0).max(0.0);
                        this.pattern_scroll
                            .set_offset(point(px(target_offset), px(0.)));

                        cx.notify();
                        true
                    })
                    .unwrap_or(false);

                if !keep_going {
                    break;
                }
            }
        });
        self._tick_task = Some(task);
    }

    pub fn stop_tick(&mut self) {
        self._tick_task = None;
    }

    pub fn toggle_play(&mut self, cx: &mut Context<Self>) {
        self.playing = !self.playing;
        if self.playing {
            self.start_tick(cx);
        } else {
            self.stop_tick();
        }
    }

    pub fn scrub_to(&mut self, window_x: f32) {
        // Ruler events use window coordinates; add the horizontal scroll offset so
        // clicking at the correct bar works even when the view is scrolled right.
        let scroll_x = f32::from(self.pattern_scroll.offset().x);
        let beat = ((window_x - HEADER_W + scroll_x) / BEAT_PX).max(0.0);
        let total = (self.total_bars * self.beats_per_bar) as f32;
        self.play_pos = beat.min(total - 0.001);
    }

    pub fn apply_drag(&mut self, mouse_x: f32) {
        let Some(ref drag) = self.drag.clone() else {
            return;
        };
        let delta = ((mouse_x - drag.mouse_start_x) / BEAT_PX).round() as isize;
        let total = (self.total_bars * self.beats_per_bar) as isize;
        let pat = &mut self.tracks[drag.track_idx].patterns[drag.pat_idx];
        match drag.kind {
            DragKind::Move => {
                pat.start = (drag.orig_start as isize + delta)
                    .max(0)
                    .min(total - drag.orig_len as isize) as usize;
            }
            DragKind::Resize => {
                pat.len = (drag.orig_len as isize + delta).max(1) as usize;
            }
        }
    }

    pub fn add_track(&mut self) {
        let idx = self.tracks.len() % TRACK_COLORS.len();
        self.tracks.push(Track {
            name: format!("Track {}", self.tracks.len() + 1),
            color: TRACK_COLORS[idx],
            muted: false,
            sample: SampleType::HiHat,
            patterns: vec![],
        });
    }

    /// Return the index of the pattern on `track` that contains `play_pos`, if any.
    pub fn active_pattern_idx(track: &crate::models::Track, play_pos: f32) -> Option<usize> {
        track.patterns.iter().position(|pat| {
            let start = pat.start as f32;
            let end = (pat.start + pat.len) as f32;
            play_pos >= start && play_pos < end
        })
    }
}
