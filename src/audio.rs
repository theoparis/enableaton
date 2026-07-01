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
}

// ── Instrument plugin trait ───────────────────────────────────────────────────

pub trait InstrumentPlugin: Send + Sync {
    fn short_label(&self) -> &str;
    fn badge_color(&self) -> u32;
    /// Synthesize one note at `midi_pitch` (A4 = 69 = 440 Hz).
    fn render_note(&self, midi_pitch: u8) -> Box<dyn Source<Item = f32> + Send>;
}

// ── Wavetable synthesizer ─────────────────────────────────────────────────────

pub enum WaveShape {
    Sine,
    Saw,
    Square,
    Triangle,
}

const WAVETABLE_SIZE: usize = 2048;

#[allow(dead_code)]
pub struct WavetableSynth {
    pub wave_shape: WaveShape,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    /// Duration the note is "held" before the release phase begins.
    pub note_len: f32,
    pub amplitude: f32,
    label: &'static str,
    color: u32,
    table: Arc<Vec<f32>>,
}

impl WavetableSynth {
    pub fn new(shape: WaveShape, label: &'static str, color: u32) -> Self {
        let table = Arc::new(Self::build_table(&shape));
        Self {
            wave_shape: shape,
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.3,
            note_len: 0.5,
            amplitude: 0.5,
            label,
            color,
            table,
        }
    }

    pub fn with_adsr(mut self, attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        self.attack = attack;
        self.decay = decay;
        self.sustain = sustain;
        self.release = release;
        self
    }

    pub fn with_note_len(mut self, note_len: f32) -> Self {
        self.note_len = note_len;
        self
    }

    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
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

    fn render_note(&self, midi_pitch: u8) -> Box<dyn Source<Item = f32> + Send> {
        let freq = 440.0 * 2.0_f64.powf((midi_pitch as f64 - 69.0) / 12.0);
        let sr = 44100_u32;
        let total = ((self.note_len + self.release) * sr as f32) as u64;
        Box::new(WavetableSource {
            table: Arc::clone(&self.table),
            phase: 0.0,
            phase_inc: freq / sr as f64,
            current: 0,
            total,
            sample_rate: sr,
            attack: self.attack,
            decay: self.decay,
            sustain: self.sustain,
            release: self.release,
            note_len: self.note_len,
            amplitude: self.amplitude,
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

        let out = (sample * self.envelope(t) * self.amplitude).clamp(-1.0, 1.0);
        self.phase += self.phase_inc;
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
