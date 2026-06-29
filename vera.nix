# vera.nix
{ lib
, rustPlatform
, makeDesktopItem
, copyDesktopItems
, makeWrapper
, autoPatchelfHook
, pkg-config
, fontconfig
, freetype
, expat
, wayland
, libxkbcommon
, libGL
, stdenv
, zenity
}:

let
  runtimeLibs = [
    fontconfig
    freetype
    expat
    wayland
    libxkbcommon
    libGL
    stdenv.cc.cc.lib
  ];
in

rustPlatform.buildRustPackage {
  pname = "vera";
  version = "0.1.0";
  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [
    copyDesktopItems
    autoPatchelfHook
    makeWrapper
    pkg-config
  ];

  buildInputs = runtimeLibs ++ [ zenity ];   # <-- add zenity here

  desktopItems = [
    (makeDesktopItem {
      name = "vera";
      desktopName = "Vera";
      exec = "vera %F";
      icon = "vera";
      comment = "Image metadata viewer";
      categories = [ "Graphics" "Viewer" ];
      mimeTypes = [ "image/jpeg" "image/png" "image/tiff" "image/bmp" ];
      terminal = false;
    })
  ];

  postInstall = ''
    mkdir -p $out/share/icons/hicolor/scalable/apps
    cp ${./assets/Vera.svg} $out/share/icons/hicolor/scalable/apps/vera.svg

    wrapProgram $out/bin/vera \
      --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath runtimeLibs} \
      --prefix PATH : ${lib.makeBinPath [ zenity ]}
  '';

  # check both these directories
  #> /run/current-system/sw/share/applications/vera.desktop
  #> /run/current-system/sw/share/icons/hicolor/scalable/apps/vera.svg
  # if they exist mission accomplished


  meta = {
    description = "Image metadata viewer";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
  };
}