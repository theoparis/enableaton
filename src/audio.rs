use std::f64::consts::PI;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use rodio::source::Source;
use rodio::{DeviceSinkBuilder, mixer::Mixer};

// ── Sample types ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SampleType {
    Kick,
    Snare,
    HiHat,
    OpenHat,
    Bass,
    SynthLead,
    Pad,
    Clap,
}

impl SampleType {
    pub fn short_label(self) -> &'static str {
        match self {
            SampleType::Kick => "BD",
            SampleType::Snare => "SD",
            SampleType::HiHat => "HH",
            SampleType::OpenHat => "OH",
            SampleType::Bass => "BS",
            SampleType::SynthLead => "SY",
            SampleType::Pad => "PD",
            SampleType::Clap => "CP",
        }
    }

    /// Background color for the sample-type badge.
    pub fn badge_color(self) -> u32 {
        match self {
            SampleType::Kick
            | SampleType::Snare
            | SampleType::HiHat
            | SampleType::OpenHat
            | SampleType::Clap => 0x1e3550,
            SampleType::Bass => 0x1a2a6a,
            SampleType::SynthLead => 0x143d35,
            SampleType::Pad => 0x3a2050,
        }
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
            .spawn(move || {
                match DeviceSinkBuilder::open_default_sink() {
                    Ok(mut sink) => {
                        sink.log_on_drop(false);
                        let _ = tx.send(sink.mixer().clone());
                        // Keep the sink (and the audio thread) alive forever.
                        loop {
                            std::thread::sleep(Duration::from_secs(3600));
                        }
                    }
                    Err(e) => eprintln!("audio: failed to open device — {e}"),
                }
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
        self.master_vol.store(f32::to_bits(vol.clamp(0.0, 1.0)), Ordering::Relaxed);
    }

    pub fn trigger(&self, sample: SampleType) {
        let vol = f32::from_bits(self.master_vol.load(Ordering::Relaxed));
        let src: Box<dyn Source<Item = f32> + Send> = match sample {
            SampleType::Kick => Box::new(kick().amplify(vol)),
            SampleType::Snare => Box::new(snare().amplify(vol)),
            SampleType::HiHat => Box::new(hihat_closed().amplify(vol)),
            SampleType::OpenHat => Box::new(hihat_open().amplify(vol)),
            SampleType::Bass => Box::new(bass().amplify(vol)),
            SampleType::SynthLead => Box::new(synth_lead().amplify(vol)),
            SampleType::Pad => Box::new(pad().amplify(vol)),
            SampleType::Clap => Box::new(clap().amplify(vol)),
        };
        self.mixer.add(src);
    }
}

// ── Synthesis primitives ──────────────────────────────────────────────────────

/// A sound source defined by a closure `f(t_seconds) -> f32`.
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

// ── Instrument voices ─────────────────────────────────────────────────────────

/// 808-style kick: pitch-swept sine with exponential decay.
fn kick() -> impl Source<Item = f32> + Send {
    FnSource::new(0.38, |t| {
        let pitch = 110.0 * (-t * 22.0).exp() + 42.0;
        let env = (-t * 11.0).exp();
        ((2.0 * PI * pitch * t).sin() * env * 0.9) as f32
    })
}

/// Snare: short noise burst + tonal body.
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

/// Closed hi-hat: high-frequency noise, very short.
fn hihat_closed() -> impl Source<Item = f32> + Send {
    let mut rng = 0xc0ffee_u64;
    FnSource::new(0.055, move |t| {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let noise = (rng >> 33) as f64 / (1u64 << 31) as f64 - 1.0;
        let env = (-t * 90.0).exp();
        (noise * env * 0.45) as f32
    })
}

/// Open hi-hat: same but longer decay.
fn hihat_open() -> impl Source<Item = f32> + Send {
    let mut rng = 0xface_u64;
    FnSource::new(0.35, move |t| {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let noise = (rng >> 33) as f64 / (1u64 << 31) as f64 - 1.0;
        let env = (-t * 12.0).exp();
        (noise * env * 0.4) as f32
    })
}

/// Sub bass: low sine with slow decay.
fn bass() -> impl Source<Item = f32> + Send {
    FnSource::new(0.55, |t| {
        let pitch = 65.0 * (-t * 6.0).exp() + 38.0;
        let env = (-t * 4.5).exp();
        ((2.0 * PI * pitch * t).sin() * env * 0.85) as f32
    })
}

/// Synth lead: detuned sawtooth (sum of harmonics), plucky.
fn synth_lead() -> impl Source<Item = f32> + Send {
    FnSource::new(0.28, |t| {
        let f0 = 440.0_f64;
        let detune = 1.007_f64;
        let env = (-t * 12.0).exp() * 0.55;
        let mut s = 0.0_f64;
        for k in 1..=8_u32 {
            let kf = k as f64;
            s += ((2.0 * PI * f0 * kf * t).sin() + (2.0 * PI * f0 * detune * kf * t).sin() * 0.6)
                / kf;
        }
        (s * env * 0.3) as f32
    })
}

/// Pad: soft chord with attack, long release.
fn pad() -> impl Source<Item = f32> + Send {
    // C4 minor triad: C4, Eb4, G4
    let freqs = [261.63_f64, 311.13, 392.0];
    FnSource::new(1.2, move |t| {
        let attack = (t * 4.0).min(1.0);
        let release = (-t * 1.5).exp();
        let env = attack * release * 0.25;
        let s: f64 = freqs
            .iter()
            .enumerate()
            .map(|(i, &f)| {
                let amp = 1.0 - i as f64 * 0.2;
                (2.0 * PI * f * t).sin() * amp
            })
            .sum();
        (s * env) as f32
    })
}

/// Clap: layered noise bursts.
fn clap() -> impl Source<Item = f32> + Send {
    let mut rng = 0xabad1dea_u64;
    FnSource::new(0.18, move |t| {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let noise = (rng >> 33) as f64 / (1u64 << 31) as f64 - 1.0;
        // Three overlapping bursts simulating hand clap.
        let b0 = (-((t - 0.000) * 60.0).powi(2)).exp();
        let b1 = (-((t - 0.012) * 55.0).powi(2)).exp();
        let b2 = (-((t - 0.026) * 40.0).powi(2)).exp();
        let env = (b0 + b1 * 0.7 + b2 * 0.5) * (-t * 18.0).exp();
        (noise * env * 0.75) as f32
    })
}
