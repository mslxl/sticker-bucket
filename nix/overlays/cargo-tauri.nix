{
  lib,
  stdenv,
  rustPlatform,
  fetchFromGitHub,
  openssl,
  pkg-config,
  glibc,
  libsoup,
  cairo,
  gtk3,
  webkitgtk,
  darwin,
}: let
  inherit (darwin.apple_sdk.frameworks) CoreServices Security SystemConfiguration;
in
  rustPlatform.buildRustPackage rec {
    pname = "tauri";
    version = "2.0.0-beta.19";

    src = fetchFromGitHub {
      owner = "tauri-apps";
      repo = "tauri";
      rev = "tauri-v${version}";
      hash = "sha256-JAa2GXsXNXSPs5niUBfpdFro7cjKVNQq+T4ptZXnr5o=";
    };

    # Manually specify the sourceRoot since this crate depends on other crates in the workspace. Relevant info at
    # https://discourse.nixos.org/t/difficulty-using-buildrustpackage-with-a-src-containing-multiple-cargo-workspaces/10202
    sourceRoot = "${src.name}/tooling/cli";

    cargoHash = "sha256-3EdtaPHCYqOhJ6cq4DDKDoXOE6je4TZRRPAOprOQB6o=";

    nativeBuildInputs = [
      pkg-config
    ];

    buildInputs =
      [
        openssl
      ]
      ++ lib.optionals stdenv.isLinux [
        glibc
        libsoup
        cairo
        gtk3
        webkitgtk
      ]
      ++ lib.optionals stdenv.isDarwin [
        CoreServices
        Security
        SystemConfiguration
      ];

    strictDeps = true;

    meta = {
      description = "Build smaller, faster, and more secure desktop applications with a web frontend";
      homepage = "https://tauri.app/";
      license = with lib.licenses; [
        asl20
        /*
        or
        */
        mit
      ];
      mainProgram = "cargo-tauri-beta";
      maintainers = with lib.maintainers; [dit7ya happysalada eveeifyeve];
    };
  }
