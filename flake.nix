{
  description = "whereami: Hyprland and Niri process viewer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        toolchain = fenix.packages.${system}.stable.toolchain;
        devToolchain = fenix.packages.${system}.combine [
          fenix.packages.${system}.stable.toolchain
          fenix.packages.${system}.stable.rust-src
          fenix.packages.${system}.stable.rust-analyzer
        ];
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        runtimeLibs = with pkgs; [
          expat
          fontconfig
          freetype
          libGL
          libX11
          libXcursor
          libXi
          libXrandr
          wayland
          libxkbcommon
        ];

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        commonArgs = {
          pname = "whereami";
          version = cargoToml.package.version;
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          strictDeps = true;

          nativeBuildInputs = with pkgs; [
            pkg-config
            autoPatchelfHook
            mold
            clang
          ];

          buildInputs =
            runtimeLibs
            ++ (with pkgs; [
              gcc.cc.lib
              glibc
            ]);
          RUSTFLAGS = "-C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.clang}/bin/clang";

        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        whereami = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
      in
      {
        packages.whereami = whereami;
        packages.default = whereami;

        devShells.default = pkgs.mkShell {
          inputsFrom = [ whereami ];
          nativeBuildInputs = [ devToolchain ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;
          RUST_LOG = "debug";
        };
      }
    );
}
