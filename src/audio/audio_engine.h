#ifndef SKIA_YOGA_UI_AUDIO_ENGINE_H_
#define SKIA_YOGA_UI_AUDIO_ENGINE_H_

#include <vector>
#include <array>
#include "ui/daw_state.h"

namespace skui::audio {

class AudioEngine {
 public:
  AudioEngine();
  ~AudioEngine();

  // Prevent copying
  AudioEngine(const AudioEngine&) = delete;
  AudioEngine& operator=(const AudioEngine&) = delete;

  bool Initialize();
  void PlayDrum(ui::DrumType type);
  void Shutdown();

  // Trigger drum sounds based on playhead step transition
  void Update(const ui::DawState& daw);

 private:
  void GenerateSamples();
  void GenerateKick(std::vector<short>& buffer);
  void GenerateSnare(std::vector<short>& buffer);
  void GenerateClosedHiHat(std::vector<short>& buffer);
  void GenerateOpenHiHat(std::vector<short>& buffer);
  void GenerateClap(std::vector<short>& buffer);
  void GenerateRim(std::vector<short>& buffer);
  void GenerateTom(std::vector<short>& buffer);
  void GenerateCrash(std::vector<short>& buffer);

  bool initialized_ = false;
  void* alc_device_ = nullptr;
  void* alc_context_ = nullptr;

  std::array<unsigned int, ui::kDrumTypeCount> buffers_{};
  std::array<unsigned int, ui::kDrumTypeCount> sources_{};

  int last_triggered_step_ = -1;
};

}  // namespace skui::audio

#endif  // SKIA_YOGA_UI_AUDIO_ENGINE_H_
