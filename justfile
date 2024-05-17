help:
    just --list

dev:
    pnpm tauri dev

build-deb: 
    pnpm tauri -b deb

build-nix:
    nix build

clean:
    rm -rf ./dist
    rm -rf ./src-tauri/target