{
  description = "whereami: Hyprland process viewer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.whereami = pkgs.rustPlatform.buildRustPackage {
          pname = "whereami";
          version = "0.1.0";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            libGL
            libxkbcommon
            wayland
          ];

          postFixup = ''
            patchelf --set-rpath "${
              pkgs.lib.makeLibraryPath [
                pkgs.libGL
                pkgs.libxkbcommon
                pkgs.wayland
              ]
            }" $out/bin/whereami
          '';
        };

        packages.default = self.packages.${system}.whereami;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            pkg-config
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.libGL
            pkgs.libxkbcommon
            pkgs.wayland
          ];

          RUST_LOG = "debug";
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      }
    );
}
