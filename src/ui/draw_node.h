#ifndef SKIA_YOGA_UI_DRAW_NODE_H_
#define SKIA_YOGA_UI_DRAW_NODE_H_

#include <functional>

#include "node.h"

namespace skui::ui {

// A node that delegates drawing to a callback function.
// Must have explicit width/height set (no self-measuring).
class DrawNode : public Node {
 public:
  using DrawCallback =
      std::function<void(SkCanvas*, float width, float height)>;

  DrawNode() = default;
  explicit DrawNode(DrawCallback callback)
      : draw_callback_(std::move(callback)) {}

  void SetDrawCallback(DrawCallback callback) {
    draw_callback_ = std::move(callback);
  }

 protected:
  void OnDraw(SkCanvas* canvas, float width, float height) override {
    if (draw_callback_) {
      draw_callback_(canvas, width, height);
    }
  }

 private:
  DrawCallback draw_callback_;
};

}  // namespace skui::ui

#endif  // SKIA_YOGA_UI_DRAW_NODE_H_
