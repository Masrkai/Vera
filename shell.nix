# shell.nix
{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    llvmPackages_21.llvm   # match the version rustc uses

    cargo
    cargo-watch
    cargo-nextest
    cargo-llvm-cov

    # ── C Libraries & Tools for Rust GUI/Crates ──────────────────
    pkg-config      # Tells Rust where to find C libraries
    fontconfig      # Required by Slint (yeslogic-fontconfig-sys)
    freetype        # Required by fontconfig
    expat           # Required by fontconfig

    # --- Windowing & UI Dependencies ---
    wayland
    libxkbcommon

    libX11
    libXcursor
    libXi
    libXrandr

    libGL

    kdePackages.kdialog
    zenity
  ];

  env = {
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
      stdenv.cc.cc.lib
      zlib
      libGL
      fontconfig
      freetype
      expat
      libxkbcommon
      libX11
      libXext
      libXrender
      glib
      dbus
      zstd
      krb5
      wayland
    ]);

    LLVM_COV      = "${pkgs.llvmPackages_21.llvm}/bin/llvm-cov";
    LLVM_PROFDATA = "${pkgs.llvmPackages_21.llvm}/bin/llvm-profdata";
  };

  shellHook = ''
    alias build='cargo build --release'
    alias test='cargo llvm-cov nextest --ignore-filename-regex="rustc-" --html'
    alias review='[ -f target/llvm-cov/html/index.html ] && xdg-open target/llvm-cov/html/index.html || { echo "No report found, run test first"; }'

    alias package-test-remote='nix-build -E "with import <nixpkgs> {}; callPackage ./default.nix {}"'
    alias package-test-local='nix-build -E "with import <nixpkgs> {}; callPackage ./default-local.nix {}"'
  '';
}
