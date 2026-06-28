#ifndef SKIA_YOGA_UI_INPUT_H_
#define SKIA_YOGA_UI_INPUT_H_

#include <functional>

namespace skui::ui {

enum class MouseButton { Left, Right, Middle };
enum class InputAction { Press, Release, Move };
enum class KeyAction { Press, Release, Repeat };

struct MouseEvent {
  float x, y;
  MouseButton button;
  InputAction action;
  bool shift = false;
  bool ctrl = false;
  bool alt = false;
};

struct ScrollEvent {
  float x, y;
  float dx, dy;
};

struct KeyEvent {
  int key;
  KeyAction action;
  bool shift = false;
  bool ctrl = false;
  bool alt = false;
};

using MouseCallback = std::function<bool(const MouseEvent&)>;
using ScrollCallback = std::function<bool(const ScrollEvent&)>;
using KeyCallback = std::function<bool(const KeyEvent&)>;

}  // namespace skui::ui

#endif  // SKIA_YOGA_UI_INPUT_H_
