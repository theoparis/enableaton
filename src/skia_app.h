#ifndef SKIA_APP_H_
#define SKIA_APP_H_

#include <functional>
#include <memory>
#include <string>

#include "include/core/SkSize.h"
#include "webgpu/webgpu_cpp.h"

#include "ui/input.h"

class SkCanvas;
class SkColorSpace;

namespace skui {

struct StatusBarWindowOptions {
  int height = 800;
  int width = 1400;
  std::string title = "Enableaton";
};

struct AppOptions {
  wgpu::BackendType backend = wgpu::BackendType::Vulkan;
  StatusBarWindowOptions window;
};

using DrawFrameCallback = std::function<void(SkCanvas*, SkISize)>;

class SkiaApp {
 public:
  explicit SkiaApp(AppOptions options = {});
  ~SkiaApp();

  SkiaApp(const SkiaApp&) = delete;
  SkiaApp& operator=(const SkiaApp&) = delete;

  int Run(const DrawFrameCallback& draw_frame);

  void SetMouseCallback(ui::MouseCallback cb);
  void SetScrollCallback(ui::ScrollCallback cb);
  void SetKeyCallback(ui::KeyCallback cb);
  wgpu::BackendType backend() const;

 private:
  struct Impl;

  AppOptions options_;
  std::unique_ptr<Impl> impl_;

  ui::MouseCallback mouse_cb_;
  ui::ScrollCallback scroll_cb_;
  ui::KeyCallback key_cb_;
};

}  // namespace skui

namespace demo {
using StatusBarWindowOptions = skui::StatusBarWindowOptions;
using AppOptions = skui::AppOptions;
using DrawFrameCallback = skui::DrawFrameCallback;
}  // namespace demo

#endif  // SKIA_APP_H_
