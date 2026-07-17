# waifu-sensor assets

- `bundles/v3` contains the authorized, converted RimoChan/waifu-sensor v3
  character vectors and the matching optimized feature weights.
- `models/ml-danbooru` contains only the model manifest and class list. The
  286 MB ONNX model is downloaded into a caller-selected cache directory and
  is intentionally not committed to this repository.

These immutable resources are compiled into the library and CLI. Applications
can load them through `BuiltinAssets`; the CLI uses them by default without
requiring repository-relative paths. Explicit file arguments remain available
for custom resources and development.

Do not put mutable SQLite databases or downloaded model binaries here. Store
databases in the embedding application's platform data directory and model
binaries in its platform cache directory. `PlatformPaths::discover` provides
the XDG/macOS/Windows-native runtime layout.
