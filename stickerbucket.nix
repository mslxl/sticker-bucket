{
  lib,
  stdenv,
  rustPlatform,
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

  libraries ? lib.throw "libraries must be specified"
}:
stdenv.mkDerivation rec {
  pname = "stickerbucket";
  version = "0.0.0";
  src = builtins.path {
    name = "source";
    path = ./.;
  };
  sourceRoot = "source/src-tauri";

  postPatch = ''
    substituteInPlace $cargoDepsCopy/libappindicator-sys-*/src/lib.rs \
    --replace-fail "libayatana-appindicator3.so.1" "${libayatana-appindicator}/lib/libayatana-appindicator3.so.1"
  '';

  pnpmDeps = pnpm.fetchDeps {
    pname = "${pname}-pnpm-deps";
    inherit src version;
    hash = "sha256-23Eph8mPnfKjKioytrAZ1tVAG6kbYx/LVMU+tL66ym0=";
  };
  pnpmRoot = "..";

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ./src-tauri/Cargo.lock;
  };

  nativeBuildInputs = [
    rustPlatform.cargoSetupHook
    cargo
    rustc
    cargo-tauri
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
          version = "0.20.2";
          src = fetchFromGitHub {
            owner = "evanw";
            repo = "esbuild";
            rev = "v${version}";
            hash = "sha256-h/Vqwax4B4nehRP9TaYbdixAZdb1hx373dNxNHvDrtY=";
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
  '';
}
