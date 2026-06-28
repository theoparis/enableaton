load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# Define dawn_repo repository rule without depending on git
def _dawn_repo_impl(repo_ctx):
    # Fetch the Dawn source via http_archive logic
    remote = repo_ctx.attr.remote
    if remote.endswith(".git"):
        remote = remote[:-4]
    url = remote + "/+archive/" + repo_ctx.attr.commit + ".tar.gz"

    repo_ctx.download_and_extract(
        url = url,
        type = "tar.gz",
    )

    # Copy the BUILD.bazel from Skia
    build_file_content = repo_ctx.read(repo_ctx.path(repo_ctx.attr.build_file))
    repo_ctx.file("BUILD.bazel", build_file_content)

    # Run the generator to create dawn_files.bzl
    generator_path = repo_ctx.path(repo_ctx.attr.generator_py)
    repo_ctx.watch(repo_ctx.attr.generator_py)

    # Resolve python paths for jinja2 and markupsafe
    jinja2_dir = str(repo_ctx.path(repo_ctx.attr.jinja2).dirname)
    markupsafe_dir = str(repo_ctx.path(repo_ctx.attr.markupsafe).dirname)
    python_path = "{}:{}".format(jinja2_dir, markupsafe_dir)

    res = repo_ctx.execute(
        ["python3", generator_path, ".", "dawn_files.bzl"],
        environment = {"PYTHONPATH": python_path},
    )
    if res.return_code != 0:
        # Fallback to python if python3 is not found
        res = repo_ctx.execute(
            ["python", generator_path, ".", "dawn_files.bzl"],
            environment = {"PYTHONPATH": python_path},
        )
        if res.return_code != 0:
            fail("Failed to generate dawn_files.bzl: " + res.stderr)

dawn_repo = repository_rule(
    implementation = _dawn_repo_impl,
    attrs = {
        "commit": attr.string(mandatory = True),
        "remote": attr.string(mandatory = True),
        "build_file": attr.label(mandatory = True),
        "generator_py": attr.label(mandatory = True),
        "jinja2": attr.label(default = "@jinja2//:BUILD.bazel"),
        "markupsafe": attr.label(default = "@markupsafe//:BUILD.bazel"),
    },
)

def _skia_ext_impl(ctx):
    # 1. Fetch Skia using http_archive (fast, cached, does not full clone on patch changes)
    http_archive(
        name = "skia",
        urls = ["https://skia.googlesource.com/skia/+archive/62efacd37737505732dbe3d8daa62abd679626a1.tar.gz"],
        type = "tar.gz",
        patches = [
            Label("//patches:skia_expose_graphite.patch"),
            Label("//patches:skia_dawn_no_arc.patch"),
        ],
        patch_args = ["-p1"],
    )

    # 2. Read deps.json from the local workspace
    json_path = ctx.path(Label("//:skia_deps.json"))
    json_content = ctx.read(json_path)
    data = json.decode(json_content)

    all_deps = {}

    for repo_data in data["direct"] + data["indirect"]:
        name = repo_data.get("name")
        if name in all_deps:
            continue
        all_deps[name] = True

        build_file = repo_data.get("build_file")
        if build_file:
            # Prefix build_file label with @skia to refer to files inside skia repo
            build_file = Label("@skia" + build_file)

        patches = repo_data.get("patches")
        if patches:
            patches = [Label("@skia" + p) for p in patches]

        if name == "dawn":
            dawn_repo(
                name = name,
                commit = repo_data.get("commit"),
                remote = repo_data.get("remote"),
                build_file = build_file,
                generator_py = Label("@skia//bazel/external/dawn:generate_dawn_files.py"),
            )
        else:
            remote = repo_data.get("remote")
            if remote.endswith(".git"):
                remote = remote[:-4]
            url = remote + "/+archive/" + repo_data.get("commit") + ".tar.gz"

            http_archive(
                name = name,
                urls = [url],
                type = "tar.gz",
                build_file = build_file,
                patch_args = repo_data.get("patch_args", []),
                patch_cmds = repo_data.get("patch_cmds"),
                patch_cmds_win = repo_data.get("patch_cmds_win"),
                patches = patches,
            )

skia_ext = module_extension(
    implementation = _skia_ext_impl,
)
