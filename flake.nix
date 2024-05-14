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
          webkitgtk
          gtk3
          cairo
          gdk-pixbuf
          glib
          dbus
          openssl
          librsvg
        ];

        packages = with pkgs; [
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
            libsoup
            webkitgtk
            librsvg
            libayatana-appindicator
        ];
      in
      {
        # Executed by `nix build`
        packages.default = pkgs.mkYarnPackage rec {
            inherit packages;
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
        apps.default = flake-utils.lib.mkApp {
            drv = packages.default;
        };

        devShell = pkgs.mkShell {
          buildInputs = packages;

          shellHook =
            ''
              export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
              export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
            '';
            # Specify the rust-src path (many editors rely on this)
            RUST_SRC_PATH = "${rust-toolchain.rust-src}/lib/rustlib/src/rust/library";
        };
      });
}