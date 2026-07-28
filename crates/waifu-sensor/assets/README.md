# waifu-sensor assets

- `bundles/v3` contains the authorized, converted RimoChan/waifu-sensor v3
  character vectors and the matching optimized feature weights.
- `models/ml-danbooru` contains the model manifest, class list, and 286 MB ONNX
  model. The model is tracked through Git LFS.

The pinned direct download URL and checksum are recorded in the workspace's
[ONNX model source index](../../../docs/onnx-model-sources.md).

These immutable resources are compiled into the library and CLI. Applications
can load them through `BuiltinAssets`; the CLI uses them by default without
requiring repository-relative paths. Explicit file arguments remain available
for custom resources and development.

Do not put mutable SQLite databases here. Store databases in the embedding
application's platform data directory. `PlatformPaths::discover` provides the
XDG/macOS/Windows-native runtime layout.
