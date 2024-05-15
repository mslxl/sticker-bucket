dev:
    pnpm tauri dev

build:
    pnpm tauri -b deb

clean:
    rm -rf ./dist
    rm -rf ./src-tauri/target