#ifndef SKIA_YOGA_UI_DAW_STATE_H_
#define SKIA_YOGA_UI_DAW_STATE_H_

#include <array>
#include <string>
#include <vector>

namespace skui::ui {

// Drum sample types for the beat sequencer
enum class DrumType {
  Kick,
  Snare,
  ClosedHiHat,
  OpenHiHat,
  Clap,
  Rim,
  Tom,
  Crash,
  kCount  // sentinel
};

constexpr int kDrumTypeCount = static_cast<int>(DrumType::kCount);

inline const char* DrumTypeName(DrumType type) {
  switch (type) {
    case DrumType::Kick:
      return "KICK";
    case DrumType::Snare:
      return "SNARE";
    case DrumType::ClosedHiHat:
      return "CH";
    case DrumType::OpenHiHat:
      return "OH";
    case DrumType::Clap:
      return "CLAP";
    case DrumType::Rim:
      return "RIM";
    case DrumType::Tom:
      return "TOM";
    case DrumType::Crash:
      return "CRASH";
    default:
      return "???";
  }
}

inline const char* DrumTypeFullName(DrumType type) {
  switch (type) {
    case DrumType::Kick:
      return "Kick Drum";
    case DrumType::Snare:
      return "Snare Drum";
    case DrumType::ClosedHiHat:
      return "Closed Hi-Hat";
    case DrumType::OpenHiHat:
      return "Open Hi-Hat";
    case DrumType::Clap:
      return "Hand Clap";
    case DrumType::Rim:
      return "Rim Shot";
    case DrumType::Tom:
      return "Tom";
    case DrumType::Crash:
      return "Crash Cymbal";
    default:
      return "Unknown";
  }
}

// Track types
enum class TrackType { Audio, Midi, DrumRack, Return, Master };

inline const char* TrackTypeName(TrackType type) {
  switch (type) {
    case TrackType::Audio:
      return "Audio";
    case TrackType::Midi:
      return "MIDI";
    case TrackType::DrumRack:
      return "Drum Rack";
    case TrackType::Return:
      return "Return";
    case TrackType::Master:
      return "Master";
    default:
      return "Unknown";
  }
}

// Number of steps in the sequencer (16-step = 1 bar of 4/4)
constexpr int kSequencerSteps = 16;
constexpr int kBeatsPerBar = 4;
constexpr int kStepsPerBeat = kSequencerSteps / kBeatsPerBar;

struct DrumPattern {
  // steps[drum_index][step_index] = velocity (0 = off, 1-127 = on)
  std::array<std::array<int, kSequencerSteps>, kDrumTypeCount> steps{};

  DrumPattern() {
    for (auto& row : steps) {
      row.fill(0);
    }
  }

  void Toggle(int drum, int step) {
    if (drum >= 0 && drum < kDrumTypeCount && step >= 0 &&
        step < kSequencerSteps) {
      steps[drum][step] = steps[drum][step] > 0 ? 0 : 100;
    }
  }

  bool IsActive(int drum, int step) const {
    if (drum >= 0 && drum < kDrumTypeCount && step >= 0 &&
        step < kSequencerSteps) {
      return steps[drum][step] > 0;
    }
    return false;
  }
};

struct Track {
  std::string name;
  TrackType type;
  int color_index = 0;
  float volume = 0.8f;
  bool muted = false;
  bool solo = false;
  bool armed = false;

  // Drum rack data (only valid if type == DrumRack)
  DrumPattern drum_pattern;
};

struct DawState {
  // Transport
  float bpm = 120.0f;
  bool playing = false;
  bool metronome_on = false;
  bool recording = false;

  // Playhead position (in steps, fractional)
  double playhead_step = 0.0;

  // Tracks
  std::vector<Track> tracks;
  int selected_track = 0;

  // Sequencer scroll position
  float timeline_scroll_x = 0.0f;

  // UI state
  bool bpm_dragging = false;
  float bpm_drag_start_y = 0.0f;
  float bpm_at_drag_start = 120.0f;

  // Bottom panel tab (for drum rack)
  enum class BottomTab { Sequencer, Mixer, Device };
  BottomTab bottom_tab = BottomTab::Sequencer;

  void InitDefaultTracks() {
    tracks.clear();

    Track drums;
    drums.name = "Drums";
    drums.type = TrackType::DrumRack;
    drums.color_index = 0;
    // Pre-fill a basic beat
    drums.drum_pattern.steps[0] = {100, 0, 0, 0, 100, 0, 0, 0,
                                   100, 0, 0, 0, 100, 0, 0, 0};  // Kick
    drums.drum_pattern.steps[1] = {0, 0, 0, 0, 100, 0, 0, 0,
                                   0, 0, 0, 0, 100, 0, 0, 0};  // Snare
    drums.drum_pattern.steps[2] = {100, 0, 100, 0, 100, 0, 100, 0,
                                   100, 0, 100, 0, 100, 0, 100, 0};  // CH
    tracks.push_back(drums);

    Track bass;
    bass.name = "Bass";
    bass.type = TrackType::Midi;
    bass.color_index = 1;
    tracks.push_back(bass);

    Track synth;
    synth.name = "Synth Lead";
    synth.type = TrackType::Midi;
    synth.color_index = 4;
    tracks.push_back(synth);

    Track pad;
    pad.name = "Pad";
    pad.type = TrackType::Audio;
    pad.color_index = 3;
    tracks.push_back(pad);

    Track fx;
    fx.name = "FX";
    fx.type = TrackType::Audio;
    fx.color_index = 6;
    tracks.push_back(fx);

    Track master;
    master.name = "Master";
    master.type = TrackType::Master;
    master.color_index = 7;
    tracks.push_back(master);
  }

  Track* SelectedTrack() {
    if (selected_track >= 0 &&
        selected_track < static_cast<int>(tracks.size())) {
      return &tracks[selected_track];
    }
    return nullptr;
  }

  bool SelectedIsDrumRack() const {
    if (selected_track >= 0 &&
        selected_track < static_cast<int>(tracks.size())) {
      return tracks[selected_track].type == TrackType::DrumRack;
    }
    return false;
  }

  void AdvancePlayhead(double delta_seconds) {
    if (!playing) return;
    double steps_per_second = (bpm / 60.0) * kStepsPerBeat;
    playhead_step += steps_per_second * delta_seconds;
    if (playhead_step >= kSequencerSteps) {
      playhead_step -= kSequencerSteps;
    }
  }
};

}  // namespace skui::ui

#endif  // SKIA_YOGA_UI_DAW_STATE_H_
