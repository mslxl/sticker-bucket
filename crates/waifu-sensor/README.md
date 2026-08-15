# waifu-sensor

Rust port of RimoChan/waifu-sensor. This package contains both the reusable
`waifu_sensor` library and the optional `waifu-sensor` command-line program. It
does not depend on the Memelith application.

The recognizer runs ML-Danbooru, projects its tag probabilities into the
optimized feature schema, and performs weighted L2 retrieval with sqlite-vec.
Multiple visual prototypes may be stored for a character, but this is an
internal detail: predictions contain each character at most once.

## Library

The caller owns the SQLite configuration and passes an open connection:

```rust,no_run
use waifu_sensor::{
    BuiltinAssets, ExecutionPolicy, MlDanbooruTagger, ModelManager, PlatformPaths,
    WaifuDatabase, WaifuSensor, rusqlite::Connection,
};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let paths = PlatformPaths::discover()?;
paths.create()?;
let bundle = BuiltinAssets::bundle()?;
let model_manifest = BuiltinAssets::model_manifest()?;
let model_classes = BuiltinAssets::model_classes()?;
let model = BuiltinAssets::model_path()?;
ModelManager::verify(&model_manifest, &model)?;
let tagger = MlDanbooruTagger::load_with_classes(
    model,
    &model_classes,
    bundle.feature_schema.clone(),
    ExecutionPolicy::Auto,
)?;
let (database, _) = WaifuDatabase::open(Connection::open(&paths.database)?, &bundle)?;
let mut sensor = WaifuSensor::new(database, tagger)?;

// Direct insertion; internal prototype grouping is not exposed.
// let id = sensor.add_character("name", &images, Default::default())?;
# Ok(())
# }
```

`prepare_character`/`commit_character` are available when feature probabilities
need review. Normal callers can use `add_character` and `add_character_images`
directly.

```rust,no_run
# use waifu_sensor::{AddCharacterOptions, WaifuSensor};
# use image::DynamicImage;
# fn review(sensor: &mut WaifuSensor, images: &[DynamicImage]) -> waifu_sensor::Result<()> {
let mut draft = sensor.prepare_character(
    "new character",
    images,
    AddCharacterOptions::default(),
)?;
// The editable vector is ordered exactly like sensor.feature_schema().features.
draft.set_feature(0, "red_hair", 1.0, sensor)?;
let character_id = sensor.commit_character(draft)?;
# let _ = character_id;
# Ok(())
# }
```

Opening a connection with `WaifuDatabase::open` is also the bundle sync step.
An empty external database is initialized. A changed revision or content digest
with the same feature schema is merged transactionally: removed bundle rows are
deactivated, new rows are added, and user characters and reference images are
preserved. A bundle character with the same name joins the existing user
character. A different feature schema is rejected explicitly because stored
user vectors cannot be converted without rerunning the tagger on their source
images.

## CLI

```sh
cargo build -p waifu-sensor --features cli --bin waifu-sensor

waifu-sensor db sync

waifu-sensor character add \
  --name "new character" image-1.png image-2.png

waifu-sensor why-not image.png "tomari mari"

waifu-sensor character add-images \
  --character "tomari mari" another-image.png
```

The CLI uses the default bundle, model manifest, class list, and ONNX model from
this crate's `assets` directory. `--bundle` and `--model-manifest` are optional
development or custom-resource overrides; a custom manifest loads both its
class list and model filename relative to the manifest directory. The model is
verified before opening the sensor, and a missing or invalid file fails without
network access. `--database` overrides the mutable platform-native database
location. Run `waifu-sensor paths` to inspect the effective data paths.

Human-facing commands select characters by their case-insensitive canonical
name. The CLI emits only human-readable text and does not expose internal UUIDs.

`train-features` consumes separately tagged training and validation JSON data.
It computes character centroids and performs sequential Bayesian optimization
of tag weights before removing dimensions that do not improve validation
accuracy. This mirrors the upstream training/validation workflow instead of
using a manually selected runtime tag list.

## Asset placement

Keep immutable, versioned inputs in this crate:

- `assets/bundles/<revision>/`: manifest, optimized feature schema, and compressed
  character vectors.
- `assets/models/<model>/`: checksum/source manifest and class list. The ONNX
  model is downloaded by the GUI into application data at runtime.
- `assets/fixtures/`: small test-only images.

Mutable SQLite data remains in the host application's platform data directory.
`PlatformPaths::discover` and the CLI use this layout:

| Platform | Database root |
| --- | --- |
| Linux | `$XDG_DATA_HOME/waifu-sensor` |
| macOS | `~/Library/Application Support/waifu-sensor` |
| Windows | `%LOCALAPPDATA%\\waifu-sensor` |

When an XDG variable is unset, the platform library applies the standard XDG
fallback. The database filename is `waifu-sensor.sqlite3`. The CLI loads bundle
data in this order:

1. an explicit `--bundle` directory;
2. the platform data directory's `bundles/current` directory when it contains a
   `manifest.json`;
3. the bundle compiled into the program.

If an installed or explicit bundle is present but invalid, loading fails rather
than silently falling back to another bundle. This prevents accidental database
synchronization against stale data.

The low-level database API still receives an already-open
`rusqlite::Connection`, so embedding applications retain complete control.
