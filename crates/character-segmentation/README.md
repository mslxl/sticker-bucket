# character-segmentation

Local Rust character instance segmentation using two ONNX models together:

- AnimeInsSeg RTMDet-Ins for anime-style character instances.
- YOLO11n-seg, filtered to the `person` class by default.

The library owns both ONNX Runtime sessions and runs them concurrently for each
input image. The CLI preserves both models' independent predictions and also
writes a merged result. Cross-model bounding-box NMS removes duplicates while
retaining the higher-confidence instance and its original mask.

The tested local models are:

```text
/Users/mslxl/Library/Caches/memelith/models/animeinsseg/rtmdetl_e60.raw.640.onnx
/Users/mslxl/Library/Caches/memelith/models/yolo11n-seg.onnx
```

Model files are not stored in Git. AnimeInsSeg uses a float32 BGR
`[1, 3, 640, 640]` input in the `0..255` range. The standard Ultralytics YOLO11
export uses float32 RGB normalized to `0..1`.

## CLI

```sh
cargo run -p character-segmentation --release -- \
  --anime-model /models/rtmdetl_e60.raw.640.onnx \
  --yolo-model /models/yolo11n-seg.onnx \
  --image characters.png \
  --output-dir output \
  --merge-iou 0.6
```

The command writes:

```text
output/
├── summary.json
├── animeinsseg/
│   ├── instances.json
│   ├── overlay.png
│   ├── instance-000-mask.png
│   └── instance-000.png
├── yolo11n-seg/
│   ├── instances.json
│   ├── overlay.png
│   ├── instance-000-person-mask.png
│   └── instance-000-person.png
└── merged/
    ├── instances.json
    ├── overlay.png
    ├── instance-000-yolo11-person-mask.png
    └── instance-000-yolo11-person.png
```

AnimeInsSeg uses a minimum confidence of `0.6` by default; override it with
`--anime-confidence` when needed.

Use `--yolo-class-label` to select another exact YOLO label. Fine-tuned YOLO
exports can provide their class order through `--yolo-labels`. Only the filtered
YOLO class participates in cross-model NMS, so unrelated object classes cannot
suppress an AnimeInsSeg character. `--merge-iou` controls the duplicate overlap
threshold and defaults to `0.6`.

The same package retains a standalone YOLO11 CLI for model-specific testing:

```sh
cargo run -p character-segmentation --bin yolo11-seg -- \
  --model /models/yolo11n-seg.onnx \
  --image group-photo.png \
  --output-dir output \
  --class-label person
```

## Library

```rust,no_run
use character_segmentation::{
    merge_segmentations, AnimeInsSegOptions, DualModelSegmenter,
    Yolo11SegmentationOptions,
};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut segmenter = DualModelSegmenter::load(
    "/models/rtmdetl_e60.raw.640.onnx",
    "/models/yolo11n-seg.onnx",
    AnimeInsSegOptions::default(),
    Yolo11SegmentationOptions::default(),
)?;
let image = character_segmentation::image::open("characters.png")?;
let results = segmenter.segment(&image)?;

println!("AnimeInsSeg: {}", results.animeinsseg.len());
println!("YOLO11: {}", results.yolo11.len());
let yolo_people = results
    .yolo11
    .into_iter()
    .filter(|instance| instance.class_name == "person")
    .collect::<Vec<_>>();
let merged = merge_segmentations(&results.animeinsseg, &yolo_people, 0.6)?;
println!("Merged: {}", merged.len());
# Ok(())
# }
```

Library callers should filter `results.yolo11` to the desired character class
before calling `merge_segmentations`. The function sorts both backends by
confidence and suppresses each lower-confidence box whose IoU is greater than
the configured threshold.

The AnimeInsSeg backend implements MMDetection-compatible box decoding, NMS,
and the 169-parameter dynamic mask head without requiring Python, PyTorch,
MMCV, or MMDetection at runtime. The integrated `character_segmentation::yolo11`
module exposes YOLO-specific `BoundingBox`, `Segmentation`,
`SegmentationOptions`, and `Yolo11Seg` types when only that backend is needed.
