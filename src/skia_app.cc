#include "skia_app.h"
#include "skia_app_internal.h"

#include "ui/input.h"

#include <GLFW/glfw3.h>

#include <algorithm>
#include <array>
#include <chrono>
#include <iostream>
#include <memory>
#include <optional>
#include <string>
#include <thread>
#include <tuple>
#include <utility>
#include <vector>

#include "dawn/dawn_proc.h"
#include "dawn/native/DawnNative.h"
#include "include/core/SkCanvas.h"
#include "include/core/SkColorSpace.h"
#include "include/core/SkSurface.h"
#include "include/gpu/graphite/BackendTexture.h"
#include "include/gpu/graphite/Context.h"
#include "include/gpu/graphite/ContextOptions.h"
#include "include/gpu/graphite/Recorder.h"
#include "include/gpu/graphite/Recording.h"
#include "include/gpu/graphite/Surface.h"
#include "include/gpu/graphite/dawn/DawnBackendContext.h"
#include "include/gpu/graphite/dawn/DawnGraphiteTypes.h"

namespace skui {
namespace {

using Clock = std::chrono::steady_clock;

wgpu::BackendType DefaultBackend() {
#if defined(__APPLE__)
    return wgpu::BackendType::Metal;
#elif defined(_WIN32)
    return wgpu::BackendType::D3D12;
#else
    return wgpu::BackendType::Vulkan;
#endif
}

bool BackendMatchesSelection(wgpu::BackendType requested, WGPUBackendType actual) {
    if (requested == wgpu::BackendType::Undefined) {
        return true;
    }
    return static_cast<WGPUBackendType>(requested) == actual;
}

std::string SurfaceStatusToString(wgpu::SurfaceGetCurrentTextureStatus status) {
    switch (status) {
        case wgpu::SurfaceGetCurrentTextureStatus::SuccessOptimal:
            return "success-optimal";
        case wgpu::SurfaceGetCurrentTextureStatus::SuccessSuboptimal:
            return "success-suboptimal";
        case wgpu::SurfaceGetCurrentTextureStatus::Timeout:
            return "timeout";
        case wgpu::SurfaceGetCurrentTextureStatus::Outdated:
            return "outdated";
        case wgpu::SurfaceGetCurrentTextureStatus::Lost:
            return "lost";
        case wgpu::SurfaceGetCurrentTextureStatus::Error:
            return "error";
    }
    return "unknown";
}

std::optional<wgpu::TextureFormat> PickSurfaceFormat(const wgpu::SurfaceCapabilities& capabilities) {
    constexpr std::array<wgpu::TextureFormat, 4> kPreferredFormats = {
        wgpu::TextureFormat::BGRA8Unorm,
        wgpu::TextureFormat::RGBA8Unorm,
        wgpu::TextureFormat::BGRA8UnormSrgb,
        wgpu::TextureFormat::RGBA8UnormSrgb,
    };

    for (wgpu::TextureFormat preferred : kPreferredFormats) {
        for (size_t i = 0; i < capabilities.formatCount; ++i) {
            if (capabilities.formats[i] == preferred) {
                return preferred;
            }
        }
    }

    if (capabilities.formatCount == 0) {
        return std::nullopt;
    }
    return capabilities.formats[0];
}

}  // namespace

struct SkiaApp::Impl {
    explicit Impl(const AppOptions& options)
        : options(options),
          instance(MakeInstance()),
          color_space(SkColorSpace::MakeSRGB()) {}

    ~Impl() { TearDown(); }

    bool Initialize() {
        if (!glfwInit()) {
            std::cerr << "Failed to initialize GLFW" << std::endl;
            return false;
        }

        if (!CreateWindow()) {
            return false;
        }
        if (!CreateSurface()) {
            return false;
        }
        if (!CreateDeviceAndContext()) {
            return false;
        }
        ConfigureSurface();
        needs_redraw = true;
        return true;
    }

    int Run(const DrawFrameCallback& draw_frame) {
        if (!Initialize()) {
            return EXIT_FAILURE;
        }

        while (!glfwWindowShouldClose(window) && !should_exit) {
            glfwPollEvents();

            int cur_width, cur_height;
            glfwGetFramebufferSize(window, &cur_width, &cur_height);
            if (cur_width > 0 && cur_height > 0 && 
                (cur_width != width_pixels || cur_height != height_pixels)) {
                width_pixels = cur_width;
                height_pixels = cur_height;
                ConfigureSurface();
                needs_redraw = true;
            }

            needs_redraw = true;

            if (needs_redraw && !RenderFrame(draw_frame)) {
                return EXIT_FAILURE;
            }

            std::this_thread::sleep_for(std::chrono::milliseconds(16));
        }

        return EXIT_SUCCESS;
    }

    bool RenderFrame(const DrawFrameCallback& draw_frame) {
        wgpu::SurfaceTexture surface_texture;
        surface.GetCurrentTexture(&surface_texture);
        const auto status = surface_texture.status;
        if (status == wgpu::SurfaceGetCurrentTextureStatus::Outdated ||
            status == wgpu::SurfaceGetCurrentTextureStatus::Lost) {
            ConfigureSurface();
            return true;
        }
        if (status != wgpu::SurfaceGetCurrentTextureStatus::SuccessOptimal &&
            status != wgpu::SurfaceGetCurrentTextureStatus::SuccessSuboptimal) {
            std::cerr << "Failed to acquire surface texture: "
                      << SurfaceStatusToString(status) << std::endl;
            return false;
        }

        wgpu::Texture texture = surface_texture.texture;
        auto backend_texture = skgpu::graphite::BackendTextures::MakeDawn(texture.Get());
        sk_sp<SkSurface> sk_surface = SkSurfaces::WrapBackendTexture(
            recorder.get(), backend_texture, color_space, nullptr);
        if (!sk_surface) {
            std::cerr << "Failed to wrap Dawn surface texture for Skia" << std::endl;
            return false;
        }

        // Determine content scale for HiDPI/Retina displays.
        float x_scale = 1.0f, y_scale = 1.0f;
        if (window) {
            glfwGetWindowContentScale(window, &x_scale, &y_scale);
        }
        SkCanvas* canvas = sk_surface->getCanvas();
        canvas->save();
        canvas->scale(x_scale, y_scale);
        int logical_w = static_cast<int>(width_pixels / x_scale);
        int logical_h = static_cast<int>(height_pixels / y_scale);
        draw_frame(canvas, SkISize::Make(logical_w, logical_h));
        canvas->restore();

        std::unique_ptr<skgpu::graphite::Recording> recording = recorder->snap();
        if (!recording) {
            std::cerr << "Failed to snap Graphite recording" << std::endl;
            return false;
        }
        if (!context->insertRecording({recording.get()})) {
            std::cerr << "Failed to submit Graphite recording" << std::endl;
            return false;
        }
        if (!context->submit()) {
            std::cerr << "Failed to flush Graphite work" << std::endl;
            return false;
        }
        if (!surface.Present()) {
            std::cerr << "Failed to present Dawn surface" << std::endl;
            return false;
        }

        needs_redraw = false;
        return true;
    }

    bool CreateWindow() {
        glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
        if (options.window.height > 0 && options.window.height < 100) {
            glfwWindowHint(GLFW_DECORATED, GLFW_FALSE);
        }

        int width = options.window.width;
        if (width <= 0) {
            GLFWmonitor* monitor = glfwGetPrimaryMonitor();
            if (monitor) {
                const GLFWvidmode* mode = glfwGetVideoMode(monitor);
                if (mode) {
                    width = mode->width;
                }
            }
        }
        if (width <= 0) {
            width = 800;
        }

        int height = options.window.height > 0 ? options.window.height : 600;

        window = glfwCreateWindow(width, height, options.window.title.c_str(), nullptr, nullptr);
        if (!window) {
            std::cerr << "Failed to create GLFW window" << std::endl;
            return false;
        }

        width_pixels = width;
        height_pixels = height;

        glfwSetWindowUserPointer(window, this);

        glfwSetWindowRefreshCallback(window, [](GLFWwindow* w) {
            auto* impl = static_cast<Impl*>(glfwGetWindowUserPointer(w));
            impl->needs_redraw = true;
        });

        glfwSetMouseButtonCallback(window, [](GLFWwindow* w, int button, int action, int mods) {
            auto* impl = static_cast<Impl*>(glfwGetWindowUserPointer(w));
            if (impl->mouse_cb) {
                double mx, my;
                glfwGetCursorPos(w, &mx, &my);
                ui::MouseEvent event{};
                event.x = static_cast<float>(mx);
                event.y = static_cast<float>(my);
                event.button = button == GLFW_MOUSE_BUTTON_LEFT ? ui::MouseButton::Left
                             : button == GLFW_MOUSE_BUTTON_RIGHT ? ui::MouseButton::Right
                             : ui::MouseButton::Middle;
                event.action = action == GLFW_PRESS ? ui::InputAction::Press : ui::InputAction::Release;
                event.shift = (mods & GLFW_MOD_SHIFT) != 0;
                event.ctrl = (mods & GLFW_MOD_CONTROL) != 0;
                event.alt = (mods & GLFW_MOD_ALT) != 0;
                impl->mouse_cb(event);
                impl->needs_redraw = true;
            }
        });

        glfwSetCursorPosCallback(window, [](GLFWwindow* w, double xpos, double ypos) {
            auto* impl = static_cast<Impl*>(glfwGetWindowUserPointer(w));
            if (impl->mouse_cb) {
                ui::MouseEvent event{};
                event.x = static_cast<float>(xpos);
                event.y = static_cast<float>(ypos);
                event.action = ui::InputAction::Move;
                event.button = ui::MouseButton::Left;  // unknown during move
                impl->mouse_cb(event);
                impl->needs_redraw = true;
            }
        });

        glfwSetScrollCallback(window, [](GLFWwindow* w, double xoffset, double yoffset) {
            auto* impl = static_cast<Impl*>(glfwGetWindowUserPointer(w));
            if (impl->scroll_cb) {
                double mx, my;
                glfwGetCursorPos(w, &mx, &my);
                ui::ScrollEvent event{};
                event.x = static_cast<float>(mx);
                event.y = static_cast<float>(my);
                event.dx = static_cast<float>(xoffset);
                event.dy = static_cast<float>(yoffset);
                impl->scroll_cb(event);
                impl->needs_redraw = true;
            }
        });

        glfwSetKeyCallback(window, [](GLFWwindow* w, int key, int /*scancode*/, int action, int mods) {
            auto* impl = static_cast<Impl*>(glfwGetWindowUserPointer(w));
            if (impl->key_cb) {
                ui::KeyEvent event{};
                event.key = key;
                event.action = action == GLFW_PRESS ? ui::KeyAction::Press
                             : action == GLFW_RELEASE ? ui::KeyAction::Release
                             : ui::KeyAction::Repeat;
                event.shift = (mods & GLFW_MOD_SHIFT) != 0;
                event.ctrl = (mods & GLFW_MOD_CONTROL) != 0;
                event.alt = (mods & GLFW_MOD_ALT) != 0;
                impl->key_cb(event);
                impl->needs_redraw = true;
            }
        });

        return true;
    }

    bool CreateSurface() {
        surface = CreatePlatformSurface(wgpu::Instance(instance->Get()), window);
        if (!surface) {
            std::cerr << "Failed to create platform surface" << std::endl;
            return false;
        }
        return true;
    }

    bool CreateDeviceAndContext() {
        DawnProcTable backend_procs = dawn::native::GetProcs();
        dawnProcSetProcs(&backend_procs);

        wgpu::RequestAdapterOptions adapter_options{};
        adapter_options.compatibleSurface = surface;
        adapter_options.backendType = options.backend;
        adapter_options.featureLevel =
            (options.backend == wgpu::BackendType::OpenGL ||
             options.backend == wgpu::BackendType::OpenGLES)
                ? wgpu::FeatureLevel::Compatibility
                : wgpu::FeatureLevel::Core;

        static constexpr const char* kToggles[] = {
#if !defined(SK_DEBUG)
            "skip_validation",
#endif
            "disable_lazy_clear_for_mapped_at_creation_buffer",
            "allow_unsafe_apis",
            "disable_robustness",
        };
        wgpu::DawnTogglesDescriptor toggles_desc{};
        toggles_desc.enabledToggleCount = sizeof(kToggles) / sizeof(kToggles[0]);
        toggles_desc.enabledToggles = kToggles;
        adapter_options.nextInChain = &toggles_desc;

        std::vector<dawn::native::Adapter> adapters = instance->EnumerateAdapters(&adapter_options);
        if (adapters.empty()) {
            std::cerr << "No Dawn adapters available for the selected backend" << std::endl;
            return false;
        }

        std::sort(
            adapters.begin(), adapters.end(),
            [](const dawn::native::Adapter& a, const dawn::native::Adapter& b) {
                WGPUAdapterInfo info_a{};
                WGPUAdapterInfo info_b{};
                wgpuAdapterGetInfo(a.Get(), &info_a);
                wgpuAdapterGetInfo(b.Get(), &info_b);
                return std::tie(info_a.adapterType, info_a.backendType) <
                       std::tie(info_b.adapterType, info_b.backendType);
            });

        dawn::native::Adapter matched_adapter;
        for (const auto& candidate : adapters) {
            WGPUAdapterInfo info{};
            wgpuAdapterGetInfo(candidate.Get(), &info);
            if (BackendMatchesSelection(options.backend, info.backendType)) {
                matched_adapter = candidate;
                break;
            }
        }
        if (!matched_adapter) {
            std::cerr << "No Dawn adapter matched the requested backend" << std::endl;
            return false;
        }

        adapter = wgpu::Adapter(matched_adapter.Get());

        wgpu::SurfaceCapabilities capabilities;
        if (!surface.GetCapabilities(adapter, &capabilities)) {
            std::cerr << "Failed to query Dawn surface capabilities" << std::endl;
            return false;
        }
        const std::optional<wgpu::TextureFormat> format = PickSurfaceFormat(capabilities);
        if (!format.has_value()) {
            std::cerr << "Dawn surface did not expose a supported format" << std::endl;
            return false;
        }
        surface_format = *format;

        wgpu::DeviceDescriptor device_desc{};
        device_desc.nextInChain = &toggles_desc;

        wgpu::Limits limits = {};
        adapter.GetLimits(&limits);
        device_desc.requiredLimits = &limits;

        device_desc.SetDeviceLostCallback(
            wgpu::CallbackMode::AllowSpontaneous,
            [](const wgpu::Device&, wgpu::DeviceLostReason reason, wgpu::StringView message) {
                if (reason != wgpu::DeviceLostReason::Destroyed) {
                    std::cerr << "Device lost: " << std::string(message.data, message.length)
                              << std::endl;
                }
            });
        device_desc.SetUncapturedErrorCallback(
            [](const wgpu::Device&, wgpu::ErrorType, wgpu::StringView message) {
                std::cerr << "Device error: " << std::string(message.data, message.length)
                          << std::endl;
            });

        device = adapter.CreateDevice(&device_desc);
        if (!device) {
            std::cerr << "Failed to create Dawn device" << std::endl;
            return false;
        }

        skgpu::graphite::DawnBackendContext backend_context;
        backend_context.fInstance = wgpu::Instance(instance->Get());
        backend_context.fDevice = device;
        backend_context.fQueue = device.GetQueue();

        skgpu::graphite::ContextOptions context_options;
        context = skgpu::graphite::ContextFactory::MakeDawn(backend_context, context_options);
        if (!context) {
            std::cerr << "Failed to create Graphite Dawn context" << std::endl;
            return false;
        }

        recorder = context->makeRecorder();
        if (!recorder) {
            std::cerr << "Failed to create Graphite recorder" << std::endl;
            return false;
        }

        return true;
    }

    void ConfigureSurface() {
        wgpu::SurfaceConfiguration config{};
        config.device = device;
        config.format = surface_format;
        config.usage = wgpu::TextureUsage::RenderAttachment | wgpu::TextureUsage::TextureBinding |
                       wgpu::TextureUsage::CopySrc | wgpu::TextureUsage::CopyDst;
        config.width = width_pixels;
        config.height = height_pixels;
        config.alphaMode = wgpu::CompositeAlphaMode::Opaque;
        config.presentMode = wgpu::PresentMode::Fifo;
        surface.Configure(&config);
    }

    void TearDown() {
        recorder.reset();
        context.reset();
        surface = nullptr;
        adapter = nullptr;
        device = nullptr;
        instance.reset();

        if (window != nullptr) {
            glfwDestroyWindow(window);
            window = nullptr;
        }
        glfwTerminate();
    }

    static std::unique_ptr<dawn::native::Instance> MakeInstance() {
        DawnProcTable backend_procs = dawn::native::GetProcs();
        dawnProcSetProcs(&backend_procs);

        wgpu::InstanceDescriptor desc{};
        static const auto kTimedWaitAny = wgpu::InstanceFeatureName::TimedWaitAny;
        desc.requiredFeatureCount = 1;
        desc.requiredFeatures = &kTimedWaitAny;
        return std::make_unique<dawn::native::Instance>(&desc);
    }

    AppOptions options;
    std::unique_ptr<dawn::native::Instance> instance;
    GLFWwindow* window = nullptr;
    int width_pixels = 0;
    int height_pixels = 0;
    wgpu::Surface surface;
    wgpu::Adapter adapter;
    wgpu::Device device;
    wgpu::TextureFormat surface_format = wgpu::TextureFormat::Undefined;
    std::unique_ptr<skgpu::graphite::Context> context;
    std::unique_ptr<skgpu::graphite::Recorder> recorder;
    sk_sp<SkColorSpace> color_space;
    bool needs_redraw = false;
    bool should_exit = false;
    Clock::time_point next_frame_deadline = Clock::now();

    ui::MouseCallback mouse_cb;
    ui::ScrollCallback scroll_cb;
    ui::KeyCallback key_cb;
};

SkiaApp::SkiaApp(AppOptions options) : options_(std::move(options)) {
    if (options_.backend == wgpu::BackendType::Undefined) {
        options_.backend = DefaultBackend();
    }
}

SkiaApp::~SkiaApp() = default;

int SkiaApp::Run(const DrawFrameCallback& draw_frame) {
    impl_ = std::make_unique<Impl>(options_);
    impl_->mouse_cb = mouse_cb_;
    impl_->scroll_cb = scroll_cb_;
    impl_->key_cb = key_cb_;
    return impl_->Run(draw_frame);
}

wgpu::BackendType SkiaApp::backend() const {
    return options_.backend;
}

void SkiaApp::SetMouseCallback(ui::MouseCallback cb) { mouse_cb_ = std::move(cb); }
void SkiaApp::SetScrollCallback(ui::ScrollCallback cb) { scroll_cb_ = std::move(cb); }
void SkiaApp::SetKeyCallback(ui::KeyCallback cb) { key_cb_ = std::move(cb); }

}  // namespace skui

#if !defined(__APPLE__)
#if defined(_WIN32)
#define GLFW_EXPOSE_NATIVE_WIN32
#else
#define GLFW_EXPOSE_NATIVE_X11
#endif
#include <GLFW/glfw3.h>
#include <GLFW/glfw3native.h>

namespace skui {
wgpu::Surface CreatePlatformSurface(wgpu::Instance instance, GLFWwindow* window) {
#if defined(_WIN32)
    wgpu::SurfaceSourceWin32HWND source;
    source.hinstance = GetModuleHandle(nullptr);
    source.hwnd = glfwGetWin32Window(window);

    wgpu::SurfaceDescriptor descriptor;
    descriptor.nextInChain = &source;

    return instance.CreateSurface(&descriptor);
#else
    wgpu::SurfaceSourceXlibWindow source;
    source.display = glfwGetX11Display();
    source.window = glfwGetX11Window(window);

    wgpu::SurfaceDescriptor descriptor;
    descriptor.nextInChain = &source;

    return instance.CreateSurface(&descriptor);
#endif
}
}  // namespace skui
#endif
