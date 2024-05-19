{
  lib,
  stdenv,
  stdenvNoCC,
  rustPlatform,
  cargo,
  esbuild,
  buildGoModule,
  fetchFromGitHub,
  cargo-tauri,
  wrapGAppsHook3,
  nodejs,
  pkg-config,
  rustc,
  gtk3,
  cairo,
  gdk-pixbuf,
  glib,
  librsvg,
  dbus,
  openssl,
  webkitgtk_4_1,
  jq,
  moreutils,
  cacert,
  libayatana-appindicator,
}:
stdenv.mkDerivation rec {
  pname = "stickybucket";
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

  pnpm-deps = stdenvNoCC.mkDerivation {
    pname = "${pname}-pnpm-deps";
    inherit src version;

    nativeBuildInputs = [
      jq
      moreutils
      nodejs.pkgs.pnpm
      cacert
    ];

    installPhase = ''
      export HOME=$(mktemp -d)
      pnpm config set store-dir $out
      # use --ignore-script and --no-optional to avoid downloading binaries
      # use --frozen-lockfile to avoid checking git deps
      pnpm install --frozen-lockfile --no-optional --ignore-script

      # Remove timestamp and sort the json files
      rm -rf $out/v3/tmp
      for f in $(find $out -name "*.json"); do
          sed -i -E -e 's/"checkedAt":[0-9]+,//g' $f
          jq --sort-keys . $f | sponge $f
      done
    '';

    dontFixup = true;
    outputHashMode = "recursive";
    outputHash = "sha256-kVemCQjMvnYdw4GMSWNb8+iosSNpAgMeNUlmjxNgafc=";
  };
  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ./src-tauri/Cargo.lock;
  };

  nativeBuildInputs = [
    rustPlatform.cargoSetupHook
    cargo
    rustc
    cargo-tauri
    wrapGAppsHook3
    nodejs.pkgs.pnpm
    pkg-config
  ];

  buildInputs = [
    webkitgtk_4_1
    gtk3
    cairo
    gdk-pixbuf
    glib
    dbus
    openssl
    librsvg
  ];
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

  preBuild = ''
    export HOME=$(mktemp -d)
    pnpm config set store-dir ${pnpm-deps}
    chmod +w ..
    pnpm install --offline --frozen-lockfile --no-optional --ignore-script
    chmod -R +w ../node_modules
    pnpm rebuild
    # Use cargo-tauri from nixpkgs instead of pnpm tauri from npm
    cargo tauri build -b deb
  '';

  preInstall = ''
    mv target/release/bundle/deb/*/data/usr/ $out
  '';
}
