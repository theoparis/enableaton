#ifndef SKIA_APP_INTERNAL_H_
#define SKIA_APP_INTERNAL_H_

#include "webgpu/webgpu_cpp.h"

struct GLFWwindow;

namespace skui {
wgpu::Surface CreatePlatformSurface(wgpu::Instance instance, GLFWwindow* window);
}

#endif  // SKIA_APP_INTERNAL_H_
