def _openal_impl(ctx):
    # Download openal-soft source archive
    ctx.download_and_extract(
        url = "https://github.com/kcat/openal-soft/archive/refs/tags/1.23.1.tar.gz",
        stripPrefix = "openal-soft-1.23.1",
    )

    # Patch common/almalloc.h to fix C++20 std::to_address ambiguity
    almalloc_content = ctx.read("common/almalloc.h")
    idx = almalloc_content.find("constexpr T *to_address(T *p) noexcept")
    if idx != -1:
        tmpl_idx = almalloc_content.rfind("template<typename T>", 0, idx)
        if tmpl_idx != -1:
            end_str = "{ return to_address(p.operator->()); }"
            end_idx = almalloc_content.find(end_str, idx)
            if end_idx != -1:
                end_pos = end_idx + len(end_str)
                almalloc_content = (
                    almalloc_content[:tmpl_idx] +
                    "#if __cplusplus >= 202002L\nusing std::to_address;\n#else\n" +
                    almalloc_content[tmpl_idx:end_pos] +
                    "\n#endif" +
                    almalloc_content[end_pos:]
                )
                ctx.file("common/almalloc.h", almalloc_content)

    # Write a custom static version.h
    ctx.file("version.h", """#ifndef ALSOFT_VERSION_H
#define ALSOFT_VERSION_H

#define ALSOFT_VERSION "1.23.1"
#define ALSOFT_VERSION_NUM 1,23,1,0
#define ALSOFT_GIT_BRANCH ""
#define ALSOFT_GIT_COMMIT_HASH ""

#endif // ALSOFT_VERSION_H
""")

    # Write a custom pre-configured config.h
    ctx.file("config.h", """#ifndef ALSOFT_CONFIG_H
#define ALSOFT_CONFIG_H

#define ALSOFT_VERSION "1.23.1"

#define HAVE_STDINT_H
#define HAVE_FLOAT_H
#define HAVE_LIMITS_H
#define HAVE_STDDEF_H
#define HAVE_STDBOOL_H
#define HAVE_RESTRICT
#define RESTRICT __restrict

#if defined(__APPLE__)
#define HAVE_COREAUDIO
#elif defined(_WIN32)
#define HAVE_WASAPI
#else
#define HAVE_ALSA
#endif

#endif // ALSOFT_CONFIG_H
""")

    # Write the custom BUILD.bazel file to compile openal-soft natively
    ctx.file("BUILD.bazel", """cc_library(
    name = "openal",
    srcs = glob([
        "common/*.cpp",
        "core/*.cpp",
        "core/filters/*.cpp",
        "core/mixer/*.cpp",
        "alc/*.cpp",
        "alc/effects/*.cpp",
        "al/*.cpp",
        "al/effects/*.cpp",
        "alc/backends/base.cpp",
        "alc/backends/null.cpp",
        "alc/backends/loopback.cpp",
    ], exclude = [
        "core/rtkit.cpp",
        "core/dbus_wrap.cpp",
        "core/mixer/mixer_sse.cpp",
        "core/mixer/mixer_sse2.cpp",
        "core/mixer/mixer_sse3.cpp",
        "core/mixer/mixer_sse41.cpp",
        "core/mixer/mixer_neon.cpp",
    ]) + select({
        "@platforms//cpu:arm64": ["core/mixer/mixer_neon.cpp"],
        "@platforms//cpu:arm": ["core/mixer/mixer_neon.cpp"],
        "@platforms//cpu:x86_64": [
            "core/mixer/mixer_sse.cpp",
            "core/mixer/mixer_sse2.cpp",
            "core/mixer/mixer_sse3.cpp",
            "core/mixer/mixer_sse41.cpp",
        ],
        "//conditions:default": [],
    }) + select({
        "@platforms//os:macos": ["alc/backends/coreaudio.cpp"],
        "@platforms//os:ios": ["alc/backends/coreaudio.cpp"],
        "@platforms//os:windows": ["alc/backends/wasapi.cpp"],
        "//conditions:default": ["alc/backends/alsa.cpp"],
    }),
    hdrs = glob([
        "include/**/*.h",
        "common/**/*.h",
        "core/**/*.h",
        "alc/**/*.h",
        "al/**/*.h",
        "**/*.h",
    ]),
    copts = select({
        "@platforms//os:windows": [],
        "//conditions:default": [
            "-std=c++17",
            "-fPIC",
            "-include", "config.h",
        ],
    }),
    defines = [
        "AL_BUILD_LIBRARY",
        "AL_ALEXT_PROTOTYPES",
    ],
    includes = [
        ".",
        "include",
        "common",
        "core",
        "alc",
    ],
    linkopts = select({
        "@platforms//os:macos": [
            "-framework", "AudioToolbox",
            "-framework", "CoreAudio",
            "-framework", "CoreFoundation",
        ],
        "@platforms//os:ios": [
            "-framework", "AudioToolbox",
            "-framework", "CoreAudio",
            "-framework", "CoreFoundation",
        ],
        "//conditions:default": [],
    }),
    visibility = ["//visibility:public"],
)
""")

openal_repository = repository_rule(
    implementation = _openal_impl,
)

def _openal_ext_impl(ctx):
    openal_repository(name = "openal")

openal_ext = module_extension(
    implementation = _openal_ext_impl,
)
