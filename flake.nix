{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
            inherit system;
            overlays = [fenix.overlays.default];
        };

        rust-toolchain = pkgs.fenix.complete;

        libraries = with pkgs;[
          webkitgtk_4_1
          gtk3
          cairo
          gdk-pixbuf
          glib
          dbus
          openssl
          librsvg
        ];

        buildInputs = with pkgs; [
            yarn
            (with rust-toolchain; [
                cargo
                rustc
                rust-src
                clippy
                rustfmt
            ])

            curl
            wget
            pkg-config
            dbus
            openssl
            glib
            gtk3
            libsoup_3
            webkitgtk_4_1
            librsvg
            libayatana-appindicator
        ];
      in rec {
        # Executed by `nix build`
        packages.default = pkgs.mkYarnPackage rec {
            packages = buildInputs;
            nativeBuildInputs = with pkgs; [
                makeWrapper
            ];
            name = "stickybucket";
            src = ./.;

            buildPhase = "yarn build";

            installPhase = ''
                mkdir -p $out/bin
                mkdir -p $out/lib

                cp src-tauri/target/release/${name} $out/lib/${name}
                makeWrapper $out/lib/${name} $out/bin/${name} \
                    --argv0 "${name}" \
                    --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath libraries}"
            '';
        };

        # Executed by `nix run`
        apps = rec {
          stickybucket = {
            type = "app";
            program = "${packages.default}/bin/stickybucket";
          };
          default = stickybucket;
        };

        devShell = pkgs.mkShell {
          inherit buildInputs;

          shellHook =
            ''
              export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
              export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
            '';

            WEBKIT_DISABLE_COMPOSITING_MODE = "1";
            # Specify the rust-src path (many editors rely on this)
            RUST_SRC_PATH = "${rust-toolchain.rust-src}/lib/rustlib/src/rust/library";
        };
      });
}