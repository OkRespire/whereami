{
  pkgs ? import <nixpkgs> { },
}:
let
  libPath =
    with pkgs;
    lib.makeLibraryPath [
      libGL
      libxkbcommon
      wayland
    ];
in
{
  devShell =
    with pkgs;
    mkShell {
      buildInputs = [
        cargo
        rustc
        rust-analyzer
        expat
        fontconfig
        freetype
        freetype.dev
        libGL
        pkg-config
        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libXrandr
        wayland
        libxkbcommon
      ];

      RUST_LOG = "debug";
      RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      LD_LIBRARY_PATH = libPath;
    };
}
