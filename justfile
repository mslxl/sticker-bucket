dev: 
  pnpm tauri dev

build:
  pnpm tauri build

clean:
  rm -rf dist
  cd src-tauri && cargo clean

install: build
  $(find src-tauri/target/release/bundle/nsis/memelith*-setup.exe | head)


