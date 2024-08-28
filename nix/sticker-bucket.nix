{
  lib,
  stdenv,
  rustPlatform,
  glib,
  cargo,
  buildGoModule,
  fetchFromGitHub,
  cargo-tauri,
  esbuild,
  wrapGAppsHook3,
  nodejs,
  pkg-config,
  rustc,
  libayatana-appindicator,
  pnpm,
  llvmPackages,
  libraries ? lib.throw "libraries must be specified",
}:
stdenv.mkDerivation rec {
  pname = "sticker-bucket";
  version = "0.1.0";
  src = builtins.path {
    name = "source";
    path = ../.;
  };
  sourceRoot = "source/src-tauri";

  postPatch = ''
    substituteInPlace $cargoDepsCopy/libappindicator-sys-*/src/lib.rs \
    --replace-fail "libayatana-appindicator3.so.1" "${libayatana-appindicator}/lib/libayatana-appindicator3.so.1"
  '';

  pnpmDeps = pnpm.fetchDeps {
    pname = "${pname}-pnpm-deps";
    inherit src version;
    hash = "sha256-5oyV+XapEf+UCVe0/SQQkTqlmHFYoN+dERKkVi5D5WM=";
  };
  pnpmRoot = "..";

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ../src-tauri/Cargo.lock;
  };

  nativeBuildInputs = [
    rustPlatform.cargoSetupHook
    cargo
    cargo-tauri
    rustc
    nodejs
    llvmPackages.clang
    pnpm.configHook
    pkg-config
    wrapGAppsHook3
  ];

  buildInputs = libraries;

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
  ESBUILD_BINARY_PATH = "${lib.getExe (esbuild.override {
    buildGoModule = args:
      buildGoModule (args
        // rec {
          version = "0.21.5";
          src = fetchFromGitHub {
            owner = "evanw";
            repo = "esbuild";
            rev = "v${version}";
            hash = "sha256-FpvXWIlt67G8w3pBKZo/mcp57LunxDmRUaCU/Ne89B8=";
          };
          vendorHash = "sha256-+BfxCyg0KkDQpHt/wycy/8CTG6YBA/VJvJFhhzUnSiQ=";
        });
  })}";

  preConfigure = ''
    # pnpm.configHook has to write to .., as our sourceRoot is set to src-tauri
    # TODO: move frontend into its own drv
    chmod +w ..
  '';

  preBuild = ''
    # Use cargo-tauri from nixpkgs instead of pnpm tauri from npm
    cargo tauri build -b deb
  '';

  preInstall = ''
    mv target/release/bundle/deb/*/data/usr/ $out

    # disable composition, which prevent blank screen, but missing out on a lot of performance
    wrapProgram $out/bin/sticker-bucket --prefix PATH : ${glib}/bin --set WEBKIT_DISABLE_COMPOSITING_MODE 1
  '';
}
