#include "audio_engine.h"

#include <cmath>
#include <cstdlib>
#include <iostream>

#include "AL/al.h"
#include "AL/alc.h"

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

namespace skui::audio {

AudioEngine::AudioEngine() = default;

AudioEngine::~AudioEngine() { Shutdown(); }

bool AudioEngine::Initialize() {
  if (initialized_) return true;

  ALCdevice* device = alcOpenDevice(nullptr);
  if (!device) {
    std::cerr << "Failed to open OpenAL device" << std::endl;
    return false;
  }
  alc_device_ = device;

  ALCcontext* context = alcCreateContext(device, nullptr);
  if (!context) {
    std::cerr << "Failed to create OpenAL context" << std::endl;
    alcCloseDevice(device);
    alc_device_ = nullptr;
    return false;
  }
  alc_context_ = context;

  if (!alcMakeContextCurrent(context)) {
    std::cerr << "Failed to make OpenAL context current" << std::endl;
    alcDestroyContext(context);
    alcCloseDevice(device);
    alc_device_ = nullptr;
    alc_context_ = nullptr;
    return false;
  }

  // Clear OpenAL errors
  alGetError();

  // Generate buffers and sources
  alGenBuffers(ui::kDrumTypeCount, buffers_.data());
  alGenSources(ui::kDrumTypeCount, sources_.data());

  if (alGetError() != AL_NO_ERROR) {
    std::cerr << "Failed to generate OpenAL buffers/sources" << std::endl;
    Shutdown();
    return false;
  }

  // Set default source properties
  for (unsigned int source : sources_) {
    alSourcef(source, AL_GAIN, 1.0f);
    alSourcef(source, AL_PITCH, 1.0f);
    alSource3f(source, AL_POSITION, 0.0f, 0.0f, 0.0f);
    alSource3f(source, AL_VELOCITY, 0.0f, 0.0f, 0.0f);
    alSourcei(source, AL_LOOPING, AL_FALSE);
  }

  GenerateSamples();

  initialized_ = true;
  return true;
}

void AudioEngine::Shutdown() {
  if (alc_context_) {
    alcMakeContextCurrent(nullptr);
    alcDestroyContext(static_cast<ALCcontext*>(alc_context_));
    alc_context_ = nullptr;
  }
  if (alc_device_) {
    alcCloseDevice(static_cast<ALCdevice*>(alc_device_));
    alc_device_ = nullptr;
  }
  initialized_ = false;
}

void AudioEngine::PlayDrum(ui::DrumType type) {
  if (!initialized_) return;
  int idx = static_cast<int>(type);
  if (idx < 0 || idx >= ui::kDrumTypeCount) return;

  unsigned int source = sources_[idx];

  // Stop if already playing to retrigger
  alSourceStop(source);
  alSourcePlay(source);
}

void AudioEngine::Update(const ui::DawState& daw) {
  if (!initialized_) return;

  if (!daw.playing) {
    last_triggered_step_ = -1;
    return;
  }

  // Trigger drum sounds based on playhead step boundary transitions
  int current_step = static_cast<int>(daw.playhead_step);
  if (current_step != last_triggered_step_) {
    // We crossed into a new step! Trigger active instruments for this step
    for (const auto& track : daw.tracks) {
      if (track.type == ui::TrackType::DrumRack && !track.muted) {
        // Trigger all active drums at this step
        for (int d = 0; d < ui::kDrumTypeCount; ++d) {
          if (track.drum_pattern.IsActive(d, current_step)) {
            // Apply volume scale relative to track volume
            int idx = d;
            alSourcef(sources_[idx], AL_GAIN, track.volume);
            PlayDrum(static_cast<ui::DrumType>(d));
          }
        }
      }
    }
    last_triggered_step_ = current_step;
  }
}

void AudioEngine::GenerateSamples() {
  std::vector<short> sample_buf;

  for (int i = 0; i < ui::kDrumTypeCount; ++i) {
    sample_buf.clear();
    auto type = static_cast<ui::DrumType>(i);

    switch (type) {
      case ui::DrumType::Kick:
        GenerateKick(sample_buf);
        break;
      case ui::DrumType::Snare:
        GenerateSnare(sample_buf);
        break;
      case ui::DrumType::ClosedHiHat:
        GenerateClosedHiHat(sample_buf);
        break;
      case ui::DrumType::OpenHiHat:
        GenerateOpenHiHat(sample_buf);
        break;
      case ui::DrumType::Clap:
        GenerateClap(sample_buf);
        break;
      case ui::DrumType::Rim:
        GenerateRim(sample_buf);
        break;
      case ui::DrumType::Tom:
        GenerateTom(sample_buf);
        break;
      case ui::DrumType::Crash:
        GenerateCrash(sample_buf);
        break;
      default:
        break;
    }

    if (!sample_buf.empty()) {
      alBufferData(buffers_[i], AL_FORMAT_MONO16, sample_buf.data(),
                   sample_buf.size() * sizeof(short), 44100);
      alSourcei(sources_[i], AL_BUFFER, buffers_[i]);
    }
  }
}

void AudioEngine::GenerateKick(std::vector<short>& buffer) {
  int len = static_cast<int>(44100 * 0.15f);  // 150ms
  buffer.resize(len);
  float phase = 0.0f;
  for (int i = 0; i < len; ++i) {
    float t = static_cast<float>(i) / 44100.0f;
    float freq = 45.0f + 105.0f * std::exp(-t * 40.0f);
    phase += 2.0f * M_PI * freq / 44100.0f;
    float val = std::sin(phase) * std::exp(-t * 20.0f);
    val = std::tanh(val * 1.3f);  // Warm saturation
    buffer[i] = static_cast<short>(val * 32767.0f);
  }
}

void AudioEngine::GenerateSnare(std::vector<short>& buffer) {
  int len = static_cast<int>(44100 * 0.20f);  // 200ms
  buffer.resize(len);
  for (int i = 0; i < len; ++i) {
    float t = static_cast<float>(i) / 44100.0f;
    float tone =
        std::sin(2.0f * M_PI * 180.0f * t) * std::exp(-t * 35.0f) * 0.4f;
    float noise =
        (static_cast<float>(std::rand()) / static_cast<float>(RAND_MAX) * 2.0f -
         1.0f) *
        std::exp(-t * 15.0f) * 0.6f;
    float val = tone + noise;
    buffer[i] = static_cast<short>(val * 32767.0f);
  }
}

void AudioEngine::GenerateClosedHiHat(std::vector<short>& buffer) {
  int len = static_cast<int>(44100 * 0.04f);  // 40ms
  buffer.resize(len);
  float prev = 0.0f;
  for (int i = 0; i < len; ++i) {
    float t = static_cast<float>(i) / 44100.0f;
    float noise =
        (static_cast<float>(std::rand()) / static_cast<float>(RAND_MAX) * 2.0f -
         1.0f);
    float highpassed = noise - prev;
    prev = noise;
    float val = highpassed * std::exp(-t * 120.0f) * 0.4f;
    buffer[i] = static_cast<short>(val * 32767.0f);
  }
}

void AudioEngine::GenerateOpenHiHat(std::vector<short>& buffer) {
  int len = static_cast<int>(44100 * 0.30f);  // 300ms
  buffer.resize(len);
  float prev = 0.0f;
  for (int i = 0; i < len; ++i) {
    float t = static_cast<float>(i) / 44100.0f;
    float noise =
        (static_cast<float>(std::rand()) / static_cast<float>(RAND_MAX) * 2.0f -
         1.0f);
    float highpassed = noise - prev;
    prev = noise;
    float val = highpassed * std::exp(-t * 18.0f) * 0.35f;
    buffer[i] = static_cast<short>(val * 32767.0f);
  }
}

void AudioEngine::GenerateClap(std::vector<short>& buffer) {
  int len = static_cast<int>(44100 * 0.25f);  // 250ms
  buffer.resize(len);
  for (int i = 0; i < len; ++i) {
    float t = static_cast<float>(i) / 44100.0f;
    float val = 0.0f;
    float noise =
        (static_cast<float>(std::rand()) / static_cast<float>(RAND_MAX) * 2.0f -
         1.0f);

    if (t < 0.010f) {
      val = noise * std::exp(-t * 200.0f) * 0.4f;
    } else if (t < 0.020f) {
      float t2 = t - 0.010f;
      val = noise * std::exp(-t2 * 200.0f) * 0.4f;
    } else if (t < 0.030f) {
      float t3 = t - 0.020f;
      val = noise * std::exp(-t3 * 200.0f) * 0.4f;
    } else {
      float tm = t - 0.030f;
      val = noise * std::exp(-tm * 20.0f) * 0.5f;
    }
    buffer[i] = static_cast<short>(val * 32767.0f);
  }
}

void AudioEngine::GenerateRim(std::vector<short>& buffer) {
  int len = static_cast<int>(44100 * 0.03f);  // 30ms
  buffer.resize(len);
  for (int i = 0; i < len; ++i) {
    float t = static_cast<float>(i) / 44100.0f;
    float val = std::sin(2.0f * M_PI * 1700.0f * t) * std::exp(-t * 150.0f);
    buffer[i] = static_cast<short>(val * 32767.0f);
  }
}

void AudioEngine::GenerateTom(std::vector<short>& buffer) {
  int len = static_cast<int>(44100 * 0.25f);  // 250ms
  buffer.resize(len);
  float phase = 0.0f;
  for (int i = 0; i < len; ++i) {
    float t = static_cast<float>(i) / 44100.0f;
    float freq = 80.0f + 140.0f * std::exp(-t * 15.0f);
    phase += 2.0f * M_PI * freq / 44100.0f;
    float val = std::sin(phase) * std::exp(-t * 10.0f) * 0.8f;
    buffer[i] = static_cast<short>(val * 32767.0f);
  }
}

void AudioEngine::GenerateCrash(std::vector<short>& buffer) {
  int len = static_cast<int>(44100 * 1.20f);  // 1.2s
  buffer.resize(len);
  float prev = 0.0f;
  for (int i = 0; i < len; ++i) {
    float t = static_cast<float>(i) / 44100.0f;
    float noise =
        (static_cast<float>(std::rand()) / static_cast<float>(RAND_MAX) * 2.0f -
         1.0f);
    float highpassed = noise - prev;
    prev = noise;
    float val = highpassed * std::exp(-t * 3.5f) * 0.4f;
    buffer[i] = static_cast<short>(val * 32767.0f);
  }
}

}  // namespace skui::audio
