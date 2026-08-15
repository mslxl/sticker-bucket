# memelith-clip

`memelith-clip` provides local FP32 ONNX text and image encoders whose normalized
embeddings share the same 512-dimensional CLIP space. Storage and nearest-neighbor
search are deliberately left to the caller.

The GUI downloads the model binaries into its application-data directory on
first launch. This crate keeps only the small manifests and tokenizer metadata
in `assets/`; see `assets/README.md` for model sources and redistribution notes.

The bundled variants are:

- `ChineseClipVitBasePatch16`: OFA Chinese-CLIP ViT-B/16. The upstream model
  does not declare a license and is included for evaluation only.
- `TaiyiClipRoberta102mVitBasePatch32`: Apache-2.0 Taiyi Chinese text encoder
  paired with the MIT-licensed OpenAI CLIP ViT-B/32 image encoder.

```rust,no_run
use memelith_clip::{BuiltinModel, ClipModel, ExecutionPolicy};

# fn example(image: &image::DynamicImage) -> memelith_clip::Result<()> {
let mut model = ClipModel::load_builtin(
    BuiltinModel::ChineseClipVitBasePatch16,
    ExecutionPolicy::Auto,
)?;
let text = model.encode_text("一只猫")?;
let image = model.encode_image(image)?;
assert_eq!(text.dimension(), image.dimension());
# Ok(())
# }
```

Run the fast unit test suite with:

```sh
cargo test -p memelith-clip
```

The end-to-end model test is ignored during routine workspace checks because it
loads four large ONNX sessions. Run it explicitly after placing the downloaded
model files in the corresponding `assets/models/` directories:

```sh
cargo test -p memelith-clip --test real_models -- --ignored
```
