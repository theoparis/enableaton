#include "skia_app_internal.h"

#define GLFW_EXPOSE_NATIVE_COCOA
#include <GLFW/glfw3.h>
#include <GLFW/glfw3native.h>

#import <Cocoa/Cocoa.h>
#import <QuartzCore/CAMetalLayer.h>

namespace skui {
wgpu::Surface CreatePlatformSurface(wgpu::Instance instance, GLFWwindow* window) {
    NSWindow* nsWindow = glfwGetCocoaWindow(window);
    NSView* nsView = [nsWindow contentView];

    [nsView setWantsLayer:YES];
    CAMetalLayer* layer = [CAMetalLayer layer];
    [nsView setLayer:layer];

    wgpu::SurfaceSourceMetalLayer source;
    source.layer = (__bridge void*)layer;

    wgpu::SurfaceDescriptor descriptor;
    descriptor.nextInChain = &source;

    return instance.CreateSurface(&descriptor);
}
}  // namespace skui
