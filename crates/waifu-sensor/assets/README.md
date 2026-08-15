# waifu-sensor assets

- `bundles/v3` contains the authorized, converted RimoChan/waifu-sensor v3
  character vectors and the matching optimized feature weights.
- `models/ml-danbooru` contains the model manifest and class list. The GUI
  downloads the 286 MB ONNX model into application data at runtime.

The pinned direct download URL and checksum are recorded in the workspace's
[ONNX model source index](../../../docs/onnx-model-sources.md).

The manifest, class list, and bundle are compiled into the library and CLI.
Applications can load them through `BuiltinAssets`; the ONNX model must be
available in application data or supplied through an explicit development setup.

Do not put mutable SQLite databases here. Store databases in the embedding
application's platform data directory. `PlatformPaths::discover` provides the
XDG/macOS/Windows-native runtime layout.
