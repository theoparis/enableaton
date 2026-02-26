use std::time::{Duration, Instant};

use rodio::source::SineWave;
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player, Source};

use crate::ui::{
	ArrangementViewData, AudioClipView, MidiClipView, MidiNoteView,
	TransportState,
};

#[derive(Clone, Copy, Debug)]
struct AudioClip {
	start_beats: f32,
	duration_beats: f32,
	frequency_hz: f32,
	gain: f32,
}

#[derive(Clone, Debug)]
struct MidiClip {
	start_beats: f32,
	duration_beats: f32,
	notes: Vec<MidiNote>,
}

#[derive(Clone, Copy, Debug)]
struct MidiNote {
	start_beats: f32,
	duration_beats: f32,
	midi_note: u8,
	velocity: f32,
}

pub struct AudioEngine {
	device_sink: MixerDeviceSink,
	players: Vec<Player>,
	clips: Vec<AudioClip>,
	midi_clips: Vec<MidiClip>,
	bpm: f32,
	is_playing: bool,
	is_recording: bool,
	is_looping: bool,
	record_started_at: Option<Instant>,
	play_started_at: Option<Instant>,
	timeline_beats: f32,
}

impl AudioEngine {
	pub fn new() -> Result<Self, String> {
		let mut device_sink = DeviceSinkBuilder::open_default_sink()
			.map_err(|err| format!("audio output init failed: {err}"))?;
		device_sink.log_on_drop(false);

		Ok(Self {
			device_sink,
			players: Vec::new(),
			clips: demo_clips(),
			midi_clips: demo_midi_clips(),
			bpm: 124.0,
			is_playing: false,
			is_recording: false,
			is_looping: false,
			record_started_at: None,
			play_started_at: None,
			timeline_beats: 0.0,
		})
	}

	pub fn play(&mut self) {
		self.tick();
		if self.is_playing {
			return;
		}

		let finished = self.players.iter().all(Player::empty);
		if self.players.is_empty() || finished {
			if finished {
				self.timeline_beats = 0.0;
			}
			self.start_new_playback_from(self.timeline_beats);
			return;
		}

		for player in &self.players {
			player.play();
		}
		self.play_started_at = Some(Instant::now());
		self.is_playing = true;
	}

	pub fn pause(&mut self) {
		self.timeline_beats = self.current_playhead_beats();
		for player in &self.players {
			player.pause();
		}
		self.is_playing = false;
		self.play_started_at = None;
	}

	pub fn set_bpm(&mut self, bpm: f32) {
		let next = bpm.clamp(40.0, 240.0);
		if (next - self.bpm).abs() < f32::EPSILON {
			return;
		}

		self.timeline_beats = self.current_playhead_beats();
		self.bpm = next;
		if self.is_playing {
			self.start_new_playback_from(self.timeline_beats);
		}
	}

	pub fn bpm(&self) -> f32 {
		self.bpm
	}

	pub fn toggle_recording(&mut self) {
		if !self.is_recording {
			self.is_recording = true;
			self.record_started_at = Some(Instant::now());
			return;
		}

		self.is_recording = false;
		let raw_beats = self.record_started_at.take().map_or(4.0, |started| {
			started.elapsed().as_secs_f32() * self.beats_per_second()
		});
		let duration_beats =
			((raw_beats * 4.0).round() / 4.0).clamp(0.25, 16.0);

		let idx = self.clips.len();
		let next_start = self
			.clips
			.iter()
			.map(|clip| clip.start_beats + clip.duration_beats)
			.fold(0.0f32, f32::max);
		let frequency_hz = 220.0 + (idx % 8) as f32 * 55.0;
		self.clips.push(AudioClip {
			start_beats: next_start,
			duration_beats,
			frequency_hz,
			gain: 0.12,
		});

		if self.is_playing {
			self.timeline_beats = self.current_playhead_beats();
			self.start_new_playback_from(self.timeline_beats);
		}
	}

	pub fn stop_and_rewind(&mut self) {
		for player in self.players.drain(..) {
			player.stop();
		}
		self.is_playing = false;
		self.play_started_at = None;
		self.timeline_beats = 0.0;
	}

	pub fn toggle_looping(&mut self) {
		self.is_looping = !self.is_looping;
	}

	pub fn transport_state(&self) -> TransportState {
		let has_pending_audio =
			self.players.iter().any(|player| !player.empty());
		TransportState {
			is_playing: self.is_playing && has_pending_audio,
			is_recording: self.is_recording,
			is_looping: self.is_looping,
		}
	}

	pub fn tick(&mut self) {
		if !self.is_playing {
			return;
		}

		if self.players.iter().all(Player::empty) {
			if self.is_looping && self.total_duration_beats() > 0.0 {
				self.start_new_playback_from(0.0);
			} else {
				self.is_playing = false;
				self.play_started_at = None;
				self.timeline_beats = self.total_duration_beats();
			}
		}
	}

	pub fn view_data(&self) -> ArrangementViewData {
		let playhead_beats = self.current_playhead_beats();

		ArrangementViewData {
			transport: self.transport_state(),
			bpm: self.bpm,
			playhead_beats,
			clips: self
				.clips
				.iter()
				.map(|clip| AudioClipView {
					start_beats: clip.start_beats,
					duration_beats: clip.duration_beats,
					frequency_hz: clip.frequency_hz,
				})
				.collect(),
			midi_clips: self
				.midi_clips
				.iter()
				.map(|clip| MidiClipView {
					start_beats: clip.start_beats,
					duration_beats: clip.duration_beats,
					notes: clip
						.notes
						.iter()
						.map(|note| MidiNoteView {
							start_beats: note.start_beats,
							duration_beats: note.duration_beats,
							midi_note: note.midi_note,
						})
						.collect(),
				})
				.collect(),
		}
	}

	fn start_new_playback_from(&mut self, timeline_beats: f32) {
		for player in self.players.drain(..) {
			player.stop();
		}

		let seconds_per_beat = self.seconds_per_beat();
		for clip in &self.clips {
			let clip_end = clip.start_beats + clip.duration_beats;
			if clip_end <= timeline_beats {
				continue;
			}

			let skipped_beats = (timeline_beats - clip.start_beats).max(0.0);
			let remaining_beats =
				(clip.duration_beats - skipped_beats).max(0.01);
			let delay_beats = (clip.start_beats - timeline_beats).max(0.0);
			let player = Player::connect_new(self.device_sink.mixer());
			let source = SineWave::new(clip.frequency_hz)
				.skip_duration(Duration::from_secs_f32(
					skipped_beats * seconds_per_beat,
				))
				.take_duration(Duration::from_secs_f32(
					remaining_beats * seconds_per_beat,
				))
				.delay(Duration::from_secs_f32(delay_beats * seconds_per_beat))
				.amplify(clip.gain);
			player.append(source);
			self.players.push(player);
		}

		for clip in &self.midi_clips {
			for note in &clip.notes {
				let note_start = clip.start_beats + note.start_beats;
				let note_end = note_start + note.duration_beats;
				if note_end <= timeline_beats {
					continue;
				}

				let skipped_beats = (timeline_beats - note_start).max(0.0);
				let remaining_beats =
					(note.duration_beats - skipped_beats).max(0.02);
				let delay_beats = (note_start - timeline_beats).max(0.0);
				let frequency_hz = midi_note_to_hz(note.midi_note);
				let player = Player::connect_new(self.device_sink.mixer());
				let source = SineWave::new(frequency_hz)
					.skip_duration(Duration::from_secs_f32(
						skipped_beats * seconds_per_beat,
					))
					.take_duration(Duration::from_secs_f32(
						remaining_beats * seconds_per_beat,
					))
					.delay(Duration::from_secs_f32(
						delay_beats * seconds_per_beat,
					))
					.amplify((0.04 + note.velocity * 0.12).min(0.24));
				player.append(source);
				self.players.push(player);
			}
		}
		self.is_playing = true;
		self.timeline_beats = timeline_beats;
		self.play_started_at = Some(Instant::now());
	}

	fn total_duration_beats(&self) -> f32 {
		let audio_end = self
			.clips
			.iter()
			.map(|clip| clip.start_beats + clip.duration_beats)
			.fold(0.0, f32::max);
		let midi_end = self
			.midi_clips
			.iter()
			.flat_map(|clip| {
				clip.notes.iter().map(|note| {
					clip.start_beats + note.start_beats + note.duration_beats
				})
			})
			.fold(0.0, f32::max);
		audio_end.max(midi_end)
	}

	fn beats_per_second(&self) -> f32 {
		self.bpm / 60.0
	}

	fn seconds_per_beat(&self) -> f32 {
		60.0 / self.bpm
	}

	fn current_playhead_beats(&self) -> f32 {
		if self.is_playing {
			self.timeline_beats
				+ self.play_started_at.map_or(0.0, |started| {
					started.elapsed().as_secs_f32() * self.beats_per_second()
				})
		} else {
			self.timeline_beats
		}
	}
}

fn demo_clips() -> Vec<AudioClip> {
	vec![]
}

fn demo_midi_clips() -> Vec<MidiClip> {
	vec![
		MidiClip {
			start_beats: 0.0,
			duration_beats: 4.0,
			notes: vec![
				MidiNote {
					start_beats: 0.00,
					duration_beats: 0.50,
					midi_note: 60,
					velocity: 0.8,
				},
				MidiNote {
					start_beats: 0.50,
					duration_beats: 0.50,
					midi_note: 64,
					velocity: 0.7,
				},
				MidiNote {
					start_beats: 1.00,
					duration_beats: 0.50,
					midi_note: 67,
					velocity: 0.75,
				},
				MidiNote {
					start_beats: 1.50,
					duration_beats: 0.50,
					midi_note: 72,
					velocity: 0.65,
				},
				MidiNote {
					start_beats: 2.00,
					duration_beats: 1.00,
					midi_note: 67,
					velocity: 0.7,
				},
				MidiNote {
					start_beats: 3.00,
					duration_beats: 1.00,
					midi_note: 64,
					velocity: 0.7,
				},
			],
		},
		MidiClip {
			start_beats: 4.0,
			duration_beats: 4.0,
			notes: vec![
				MidiNote {
					start_beats: 0.00,
					duration_beats: 0.50,
					midi_note: 62,
					velocity: 0.75,
				},
				MidiNote {
					start_beats: 0.50,
					duration_beats: 0.50,
					midi_note: 65,
					velocity: 0.7,
				},
				MidiNote {
					start_beats: 1.00,
					duration_beats: 0.50,
					midi_note: 69,
					velocity: 0.8,
				},
				MidiNote {
					start_beats: 1.50,
					duration_beats: 0.50,
					midi_note: 74,
					velocity: 0.68,
				},
				MidiNote {
					start_beats: 2.00,
					duration_beats: 2.00,
					midi_note: 67,
					velocity: 0.72,
				},
			],
		},
	]
}

fn midi_note_to_hz(midi_note: u8) -> f32 {
	440.0 * (2.0f32).powf((f32::from(midi_note) - 69.0) / 12.0)
}
