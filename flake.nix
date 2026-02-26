{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs, ... }:
    let
      eachSystem = nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
        "riscv64-linux"
        "aarch64-darwin"
      ];
    in
    {
      devShells = eachSystem (
        system: with nixpkgs.legacyPackages.${system}; {
          default = mkShell rec {
            nativeBuildInputs = [
              rustc
              cargo
              rustfmt
              clippy
              rust-analyzer
              nixfmt
              nil
              pkg-config
            ]
            ++ lib.optionals stdenv.hostPlatform.isLinux [
              wild
            ];

            buildInputs = [
              vulkan-loader
            ]
            ++ lib.optionals stdenv.hostPlatform.isLinux [
              alsa-lib
              wayland
              libxkbcommon
            ];

            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
          };
        }
      );
    };
}
