# Memelith

This repository is a Rust workspace containing:

- `memelith-core`: Memelith's application core.
- `memelith-clip`: local Chinese text and image CLIP embedding with bundled
  FP32 ONNX models.
- `waifu-sensor`: an independent Rust library and optional CLI for explainable
  anime character retrieval.
- `character-segmentation`: a Rust library and CLI that runs AnimeInsSeg and
  YOLO11 segmentation models together with ONNX Runtime, merges duplicate
  detections using cross-model NMS, and includes a standalone YOLO11 CLI.
- `memelith`: a GPUI desktop application that uses `memelith-core`.

## Development

Run all checks and tests:

```sh
cargo test --workspace
```

Build the independent waifu-sensor CLI:

```sh
cargo build -p waifu-sensor --features cli --bin waifu-sensor
```

Run local YOLO11n segmentation after exporting a model to ONNX:

```sh
cargo run -p character-segmentation --bin yolo11-seg -- \
  --model /path/to/yolo11n-seg.onnx \
  --image input.png \
  --output-dir output \
  --class-label person
```

Run both local character-segmentation backends on the same image:

```sh
cargo run -p character-segmentation --release -- \
  --anime-model /path/to/rtmdetl_e60.raw.640.onnx \
  --yolo-model /path/to/yolo11n-seg.onnx \
  --image input.png \
  --output-dir output \
  --merge-iou 0.6
```

This writes the untouched `animeinsseg/` and `yolo11n-seg/` results plus a
`merged/` directory where overlapping predictions retain the higher-confidence
instance.

Start the GUI:

```sh
cargo run -p memelith
```

Build the native macOS application bundle from the workspace root:

```sh
nix develop --command scripts/package-macos.sh
```

The application is written to
`target/macos-bundle/release/bundle/osx/Memelith.app`. The script bundles
non-system dynamic libraries and applies an ad-hoc signature by default. Set
`CODESIGN_IDENTITY` to use an installed signing identity instead. The generated
arm64 bundle requires macOS 14 or later.

Bundled ONNX artifacts are stored through Git LFS. Their pinned upstream
download or export sources are listed in
[`docs/onnx-model-sources.md`](docs/onnx-model-sources.md).

The workspace enables GPUI's `runtime_shaders` feature so macOS development does
not require the Metal command-line compiler. Release packaging can disable this
feature when a full Xcode installation is available.
