def _openal_repo_impl(ctx):
    # Create wrapper header AL/al.h
    ctx.file("AL/al.h", """#ifndef OPENAL_WRAPPER_AL_H_
#define OPENAL_WRAPPER_AL_H_

#if defined(__APPLE__)
#include <OpenAL/al.h>
#else
#include_next <AL/al.h>
#endif

#endif // OPENAL_WRAPPER_AL_H_
""")

    # Create wrapper header AL/alc.h
    ctx.file("AL/alc.h", """#ifndef OPENAL_WRAPPER_ALC_H_
#define OPENAL_WRAPPER_ALC_H_

#if defined(__APPLE__)
#include <OpenAL/alc.h>
#else
#include_next <AL/alc.h>
#endif

#endif // OPENAL_WRAPPER_ALC_H_
""")

    # Create the BUILD file mapping dependencies
    ctx.file("BUILD.bazel", """cc_library(
    name = "openal",
    hdrs = [
        "AL/al.h",
        "AL/alc.h",
    ],
    includes = ["."],
    linkopts = select({
        "@platforms//os:macos": ["-framework", "OpenAL"],
        "@platforms//os:ios": ["-framework", "OpenAL"],
        "//conditions:default": ["-lopenal"],
    }),
    visibility = ["//visibility:public"],
)
""")

openal_repository = repository_rule(
    implementation = _openal_repo_impl,
)

def _openal_ext_impl(ctx):
    openal_repository(name = "openal")

openal_ext = module_extension(
    implementation = _openal_ext_impl,
)
