use std::f64::consts::PI;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use rodio::source::Source;
use rodio::{DeviceSinkBuilder, mixer::Mixer};

// ── Sample types (percussion only) ───────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SampleType {
    Kick,
    Snare,
    HiHat,
    OpenHat,
    Clap,
}

impl SampleType {
    pub fn short_label(self) -> &'static str {
        match self {
            SampleType::Kick => "BD",
            SampleType::Snare => "SD",
            SampleType::HiHat => "HH",
            SampleType::OpenHat => "OH",
            SampleType::Clap => "CP",
        }
    }

    pub fn badge_color(self) -> u32 {
        0x1e3550
    }

    pub fn all() -> &'static [SampleType] {
        &[
            SampleType::Kick,
            SampleType::Snare,
            SampleType::HiHat,
            SampleType::OpenHat,
            SampleType::Clap,
        ]
    }

    /// Next sample in the built-in bank (wraps around). Used for pad sample selection.
    pub fn next(self) -> SampleType {
        let all = Self::all();
        let idx = all.iter().position(|s| *s == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }
}

// ── Instrument plugin trait ───────────────────────────────────────────────────

/// How a parameter is displayed and edited in the device panel.
#[derive(Clone, Copy)]
pub enum ParamKind {
    /// Continuous value dragged vertically; shown with the given unit suffix.
    Float(&'static str),
    /// One of a fixed set of labelled choices; the value is the choice index.
    Choice(&'static [&'static str]),
}

pub struct ParamSpec {
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub kind: ParamKind,
    /// Section header the param is grouped under in the device panel.
    pub group: &'static str,
}

pub trait InstrumentPlugin: Send + Sync {
    fn short_label(&self) -> &str;
    fn badge_color(&self) -> u32;
    /// Synthesize one note at `midi_pitch` (A4 = 69 = 440 Hz).
    fn render_note(&self, midi_pitch: u8) -> Box<dyn Source<Item = f32> + Send>;
    /// Editable parameters, if any. Indices are stable and shared with
    /// `get_param` / `set_param`.
    fn params(&self) -> &'static [ParamSpec] {
        &[]
    }
    fn get_param(&self, _idx: usize) -> f32 {
        0.0
    }
    fn set_param(&self, _idx: usize, _val: f32) {}
}

// ── Wavetable synthesizer ─────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub enum WaveShape {
    Sine,
    Saw,
    Square,
    Triangle,
}

const WAVETABLE_SIZE: usize = 2048;
const N_SHAPES: usize = 4;

// Parameter indices — shared between the params table, getters and render_note.
pub const P_SHAPE: usize = 0;
pub const P_ATTACK: usize = 1;
pub const P_DECAY: usize = 2;
pub const P_SUSTAIN: usize = 3;
pub const P_RELEASE: usize = 4;
pub const P_NOTE_LEN: usize = 5;
pub const P_AMP: usize = 6;
pub const P_LFO_TARGET: usize = 7;
pub const P_LFO_RATE: usize = 8;
pub const P_LFO_DEPTH: usize = 9;
const N_PARAMS: usize = 10;

const SHAPE_NAMES: &[&str] = &["SIN", "SAW", "SQR", "TRI"];
const LFO_TARGETS: &[&str] = &["OFF", "PITCH", "AMP"];

const WT_PARAMS: [ParamSpec; N_PARAMS] = [
    ParamSpec { name: "WAVE", min: 0.0, max: 3.0, kind: ParamKind::Choice(SHAPE_NAMES), group: "OSC" },
    ParamSpec { name: "ATK", min: 0.001, max: 1.0, kind: ParamKind::Float("s"), group: "ENVELOPE" },
    ParamSpec { name: "DEC", min: 0.001, max: 1.0, kind: ParamKind::Float("s"), group: "ENVELOPE" },
    ParamSpec { name: "SUS", min: 0.0, max: 1.0, kind: ParamKind::Float(""), group: "ENVELOPE" },
    ParamSpec { name: "REL", min: 0.01, max: 2.0, kind: ParamKind::Float("s"), group: "ENVELOPE" },
    ParamSpec { name: "LEN", min: 0.05, max: 2.0, kind: ParamKind::Float("s"), group: "OSC" },
    ParamSpec { name: "VOL", min: 0.0, max: 1.0, kind: ParamKind::Float(""), group: "OSC" },
    ParamSpec { name: "DEST", min: 0.0, max: 2.0, kind: ParamKind::Choice(LFO_TARGETS), group: "LFO" },
    ParamSpec { name: "RATE", min: 0.1, max: 20.0, kind: ParamKind::Float("Hz"), group: "LFO" },
    ParamSpec { name: "DEPTH", min: 0.0, max: 1.0, kind: ParamKind::Float(""), group: "LFO" },
];

/// Wavetable synth with a runtime-editable parameter set (device panel edits
/// params from the UI thread while notes render on the audio thread, hence
/// the atomics).
pub struct WavetableSynth {
    params: [AtomicU32; N_PARAMS],
    label: &'static str,
    color: u32,
    tables: [Arc<Vec<f32>>; N_SHAPES],
}

impl WavetableSynth {
    pub fn new(shape: WaveShape, label: &'static str, color: u32) -> Self {
        let tables = [
            Arc::new(Self::build_table(&WaveShape::Sine)),
            Arc::new(Self::build_table(&WaveShape::Saw)),
            Arc::new(Self::build_table(&WaveShape::Square)),
            Arc::new(Self::build_table(&WaveShape::Triangle)),
        ];
        let defaults: [f32; N_PARAMS] = [
            match shape {
                WaveShape::Sine => 0.0,
                WaveShape::Saw => 1.0,
                WaveShape::Square => 2.0,
                WaveShape::Triangle => 3.0,
            },
            0.01, // attack
            0.1,  // decay
            0.7,  // sustain
            0.3,  // release
            0.5,  // note len
            0.5,  // amplitude
            0.0,  // lfo target (off)
            5.0,  // lfo rate
            0.3,  // lfo depth
        ];
        Self {
            params: defaults.map(|v| AtomicU32::new(f32::to_bits(v))),
            label,
            color,
            tables,
        }
    }

    fn get(&self, idx: usize) -> f32 {
        f32::from_bits(self.params[idx].load(Ordering::Relaxed))
    }

    fn set(&self, idx: usize, val: f32) {
        self.params[idx].store(f32::to_bits(val), Ordering::Relaxed);
    }

    pub fn with_adsr(self, attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        self.set(P_ATTACK, attack);
        self.set(P_DECAY, decay);
        self.set(P_SUSTAIN, sustain);
        self.set(P_RELEASE, release);
        self
    }

    pub fn with_note_len(self, note_len: f32) -> Self {
        self.set(P_NOTE_LEN, note_len);
        self
    }

    pub fn with_amplitude(self, amplitude: f32) -> Self {
        self.set(P_AMP, amplitude);
        self
    }

    fn build_table(shape: &WaveShape) -> Vec<f32> {
        (0..WAVETABLE_SIZE)
            .map(|i| {
                let p = i as f64 / WAVETABLE_SIZE as f64;
                match shape {
                    WaveShape::Sine => (2.0 * PI * p).sin() as f32,
                    WaveShape::Saw => (2.0 * p - 1.0) as f32,
                    WaveShape::Square => {
                        if p < 0.5 {
                            1.0
                        } else {
                            -1.0
                        }
                    }
                    WaveShape::Triangle => {
                        let t = p * 2.0;
                        if t < 1.0 {
                            (2.0 * t - 1.0) as f32
                        } else {
                            (3.0 - 2.0 * t) as f32
                        }
                    }
                }
            })
            .collect()
    }
}

impl InstrumentPlugin for WavetableSynth {
    fn short_label(&self) -> &str {
        self.label
    }

    fn badge_color(&self) -> u32 {
        self.color
    }

    fn params(&self) -> &'static [ParamSpec] {
        &WT_PARAMS
    }

    fn get_param(&self, idx: usize) -> f32 {
        self.get(idx)
    }

    fn set_param(&self, idx: usize, val: f32) {
        let spec = &WT_PARAMS[idx];
        self.set(idx, val.clamp(spec.min, spec.max));
    }

    fn render_note(&self, midi_pitch: u8) -> Box<dyn Source<Item = f32> + Send> {
        let freq = 440.0 * 2.0_f64.powf((midi_pitch as f64 - 69.0) / 12.0);
        let sr = 44100_u32;
        let note_len = self.get(P_NOTE_LEN);
        let release = self.get(P_RELEASE);
        let total = ((note_len + release) * sr as f32) as u64;
        let shape_idx = (self.get(P_SHAPE).round() as usize).min(N_SHAPES - 1);
        Box::new(WavetableSource {
            table: Arc::clone(&self.tables[shape_idx]),
            phase: 0.0,
            phase_inc: freq / sr as f64,
            current: 0,
            total,
            sample_rate: sr,
            attack: self.get(P_ATTACK),
            decay: self.get(P_DECAY),
            sustain: self.get(P_SUSTAIN),
            release,
            note_len,
            amplitude: self.get(P_AMP),
            lfo_target: self.get(P_LFO_TARGET).round() as u8,
            lfo_rate: self.get(P_LFO_RATE),
            lfo_depth: self.get(P_LFO_DEPTH),
        })
    }
}

struct WavetableSource {
    table: Arc<Vec<f32>>,
    phase: f64,
    phase_inc: f64,
    current: u64,
    total: u64,
    sample_rate: u32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    note_len: f32,
    amplitude: f32,
    /// 0 = off, 1 = pitch (vibrato), 2 = amplitude (tremolo).
    lfo_target: u8,
    lfo_rate: f32,
    lfo_depth: f32,
}

impl WavetableSource {
    fn envelope(&self, t: f32) -> f32 {
        if t < self.attack {
            t / self.attack
        } else if t < self.attack + self.decay {
            let dt = (t - self.attack) / self.decay;
            1.0 - dt * (1.0 - self.sustain)
        } else if t < self.note_len {
            self.sustain
        } else {
            let dt = ((t - self.note_len) / self.release).min(1.0);
            self.sustain * (1.0 - dt)
        }
    }
}

impl Iterator for WavetableSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.current >= self.total {
            return None;
        }
        let t = self.current as f32 / self.sample_rate as f32;

        let n = self.table.len() as f64;
        let idx = self.phase.fract() * n;
        let i0 = idx.floor() as usize % self.table.len();
        let i1 = (i0 + 1) % self.table.len();
        let frac = (idx - idx.floor()) as f32;
        let sample = self.table[i0] * (1.0 - frac) + self.table[i1] * frac;

        let lfo = (2.0 * PI * self.lfo_rate as f64 * t as f64).sin();
        let amp_mod = if self.lfo_target == 2 {
            // Tremolo: modulate down from full amplitude.
            1.0 - self.lfo_depth * 0.5 * (1.0 + lfo as f32)
        } else {
            1.0
        };

        let out = (sample * self.envelope(t) * self.amplitude * amp_mod).clamp(-1.0, 1.0);
        let pitch_mod = if self.lfo_target == 1 {
            // Vibrato: ±1 semitone at full depth.
            2.0_f64.powf(lfo * self.lfo_depth as f64 / 12.0)
        } else {
            1.0
        };
        self.phase += self.phase_inc * pitch_mod;
        self.current += 1;
        Some(out)
    }
}

impl Source for WavetableSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> std::num::NonZero<u16> {
        std::num::NonZero::new(1).unwrap()
    }
    fn sample_rate(&self) -> std::num::NonZero<u32> {
        std::num::NonZero::new(self.sample_rate).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

// ── Engine ────────────────────────────────────────────────────────────────────

pub struct AudioEngine {
    mixer: Mixer,
    master_vol: Arc<AtomicU32>,
}

impl AudioEngine {
    /// Spin up the audio thread and return an engine backed by it.
    /// Returns `None` if no audio device is available.
    pub fn start() -> Option<Self> {
        let (tx, rx) = std::sync::mpsc::channel::<Mixer>();

        std::thread::Builder::new()
            .name("audio".into())
            .spawn(move || match DeviceSinkBuilder::open_default_sink() {
                Ok(mut sink) => {
                    sink.log_on_drop(false);
                    let _ = tx.send(sink.mixer().clone());
                    loop {
                        std::thread::sleep(Duration::from_secs(3600));
                    }
                }
                Err(e) => eprintln!("audio: failed to open device — {e}"),
            })
            .ok();

        rx.recv_timeout(Duration::from_secs(2))
            .ok()
            .map(|mixer| Self {
                mixer,
                master_vol: Arc::new(AtomicU32::new(f32::to_bits(1.0))),
            })
    }

    pub fn set_master_vol(&self, vol: f32) {
        self.master_vol
            .store(f32::to_bits(vol.clamp(0.0, 1.0)), Ordering::Relaxed);
    }

    pub fn trigger(&self, sample: SampleType) {
        let vol = f32::from_bits(self.master_vol.load(Ordering::Relaxed));
        let src: Box<dyn Source<Item = f32> + Send> = match sample {
            SampleType::Kick => Box::new(kick().amplify(vol)),
            SampleType::Snare => Box::new(snare().amplify(vol)),
            SampleType::HiHat => Box::new(hihat_closed().amplify(vol)),
            SampleType::OpenHat => Box::new(hihat_open().amplify(vol)),
            SampleType::Clap => Box::new(clap().amplify(vol)),
        };
        self.mixer.add(src);
    }

    pub fn trigger_plugin(&self, plugin: &dyn InstrumentPlugin, midi_pitch: Option<u8>) {
        let pitch = midi_pitch.unwrap_or(60);
        let vol = f32::from_bits(self.master_vol.load(Ordering::Relaxed));
        let src = plugin.render_note(pitch);
        self.mixer.add(src.amplify(vol));
    }
}

// ── Synthesis primitives ──────────────────────────────────────────────────────

struct FnSource {
    sample_rate: u32,
    current: u64,
    total: u64,
    func: Box<dyn FnMut(f64) -> f32 + Send>,
}

impl FnSource {
    fn new(duration: f64, func: impl FnMut(f64) -> f32 + Send + 'static) -> Self {
        let sr = 44100u32;
        Self {
            sample_rate: sr,
            current: 0,
            total: (duration * sr as f64) as u64,
            func: Box::new(func),
        }
    }
}

impl Iterator for FnSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.current >= self.total {
            return None;
        }
        let t = self.current as f64 / self.sample_rate as f64;
        let s = (self.func)(t).clamp(-1.0, 1.0);
        self.current += 1;
        Some(s)
    }
}

impl Source for FnSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> std::num::NonZero<u16> {
        std::num::NonZero::new(1).unwrap()
    }
    fn sample_rate(&self) -> std::num::NonZero<u32> {
        std::num::NonZero::new(self.sample_rate).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

// ── Percussion voices ─────────────────────────────────────────────────────────

fn kick() -> impl Source<Item = f32> + Send {
    FnSource::new(0.38, |t| {
        let pitch = 110.0 * (-t * 22.0).exp() + 42.0;
        let env = (-t * 11.0).exp();
        ((2.0 * PI * pitch * t).sin() * env * 0.9) as f32
    })
}

fn snare() -> impl Source<Item = f32> + Send {
    let mut rng = 0xdeadbeef_u64;
    FnSource::new(0.22, move |t| {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let noise = (rng >> 33) as f64 / (1u64 << 31) as f64 - 1.0;
        let tone = (2.0 * PI * 185.0 * t).sin();
        let env_noise = (-t * 28.0).exp();
        let env_tone = (-t * 18.0).exp();
        ((noise * env_noise * 0.65 + tone * env_tone * 0.35) * 0.85) as f32
    })
}

fn hihat_closed() -> impl Source<Item = f32> + Send {
    let mut rng = 0xc0ffee_u64;
    FnSource::new(0.055, move |t| {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let noise = (rng >> 33) as f64 / (1u64 << 31) as f64 - 1.0;
        let env = (-t * 90.0).exp();
        (noise * env * 0.45) as f32
    })
}

fn hihat_open() -> impl Source<Item = f32> + Send {
    let mut rng = 0xface_u64;
    FnSource::new(0.35, move |t| {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let noise = (rng >> 33) as f64 / (1u64 << 31) as f64 - 1.0;
        let env = (-t * 12.0).exp();
        (noise * env * 0.4) as f32
    })
}

fn clap() -> impl Source<Item = f32> + Send {
    let mut rng = 0xabad1dea_u64;
    FnSource::new(0.18, move |t| {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let noise = (rng >> 33) as f64 / (1u64 << 31) as f64 - 1.0;
        let b0 = (-((t - 0.000) * 60.0).powi(2)).exp();
        let b1 = (-((t - 0.012) * 55.0).powi(2)).exp();
        let b2 = (-((t - 0.026) * 40.0).powi(2)).exp();
        let env = (b0 + b1 * 0.7 + b2 * 0.5) * (-t * 18.0).exp();
        (noise * env * 0.75) as f32
    })
}
