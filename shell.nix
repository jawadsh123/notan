let
  rustOverlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> {
    overlays = [ rustOverlay ];
  };
  rust = pkgs.rust-bin.stable.latest.default.override{
    # for rust-analyzer
    extensions = [ "rust-src" ];
    targets = ["wasm32-unknown-unknown" "aarch64-apple-ios" "x86_64-apple-ios" "aarch64-linux-android" "armv7-linux-androideabi"];
  };
in
  pkgs.mkShell {
    buildInputs = [
      rust
      pkgs.rust-analyzer
      pkgs.wasm-pack

      pkgs.cmake
      pkgs.libGL
    ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin 
    (with pkgs.darwin.apple_sdk.frameworks; [ OpenGL CoreFoundation CoreVideo AppKit ]);
    shellHook = ''
      #source $PWD/third_party/emsdk/emsdk_env.sh
    '';

    ####################################################################
    # Without  this, almost  everything  fails with  locale issues  when
    # using `nix-shell --pure` (at least on NixOS).
    # See
    # + https://github.com/NixOS/nix/issues/318#issuecomment-52986702
    # + http://lists.linuxfromscratch.org/pipermail/lfs-support/2004-June/023900.html
    ####################################################################
    LOCALE_ARCHIVE = if pkgs.stdenv.isLinux then "${pkgs.glibcLocales}/lib/locale/locale-archive" else "";

    RUST_BACKTRACE = 1;
  }
