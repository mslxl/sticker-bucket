help:
    just --list

build-nix:
  #!/usr/bin/env -S nix shell nixpkgs#bash nixpkgs#nix-output-monitor --command bash
  nix build --log-format internal-json -v |& nom --json

dev:
    pnpm tauri dev

build-deb: 
    pnpm tauri build -b deb

clean:
    rm -rf ./dist
    rm -rf ./src-tauri/target
