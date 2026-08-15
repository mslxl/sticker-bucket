# ONNX model sources

## GUI downloads

The macOS GUI does not bundle ONNX files. On first launch it downloads the three
files below into `~/Library/Application Support/memelith/models`, verifies the
exact byte size and SHA-256 digest, and only then exposes each completed file to
the application. Interrupted downloads use hidden `.part` files and restart on
the next launch.

| Cached file | Pinned download | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| `chinese-clip-vit-base-patch16/text_encoder.onnx` | [`zihuv/chinese-clip-vit-base-patch16-onnx` `text.onnx` at `8001064`][gui-clip-text] | 408,491,787 | `6d8de6ba498ae2c944ec2b578371c1b3e9dd5a9a991dd4f7dabc16b8111c6f4d` |
| `chinese-clip-vit-base-patch16/image_encoder.onnx` | [`zihuv/chinese-clip-vit-base-patch16-onnx` `visual.onnx` at `8001064`][gui-clip-image] | 344,963,070 | `303f72307943b7ef5e478ac0f8a5686bdfc62847b8b331e32e89fe223ffcfe35` |
| `ml-danbooru/ml_caformer_m36_dec-5-97527.onnx` | [`deepghs/ml-danbooru-onnx` at `eb90583`][ml-danbooru-download] | 286,584,124 | `4ea7aa66df59632c71e036cd9eda9c89e0eaf58bcd39bf9718e63583378cba75` |

The downloaded Chinese-CLIP pair was compared with the repository's local FP32
export using representative Chinese and English text plus the CLIP test image;
the normalized embeddings had cosine similarity above `0.999999`. The cached
pair uses the repository's existing `model.json`, `tokenizer.json`, and
`NOTICE.md`, which are embedded in the GUI binary because they are small.

The Chinese-CLIP upstream project did not declare a license at the pinned base
model revision. The download repository's metadata does not replace the base
model's licensing terms; do not assume redistribution or commercial-use rights.

## Development and export references

The repository does not commit ONNX binaries. The table below records where
each optional development model originated. The four CLIP files are
local FP32 exports; their upstream projects do not publish these exact ONNX
artifacts. Recreate them from the pinned Hugging Face snapshots with
[`crates/clip/tools/export_models.py`](../crates/clip/tools/export_models.py).

| Optional model | Acquisition | Upstream source |
| --- | --- | --- |
| `crates/clip/assets/models/chinese-clip-vit-base-patch16/text_encoder.onnx` | Local FP32 export | [OFA-Sys/chinese-clip-vit-base-patch16 at `36e679e`][chinese-clip] |
| `crates/clip/assets/models/chinese-clip-vit-base-patch16/image_encoder.onnx` | Local FP32 export | [OFA-Sys/chinese-clip-vit-base-patch16 at `36e679e`][chinese-clip] |
| `crates/clip/assets/models/taiyi-clip-roberta-102m-vit-base-patch32/text_encoder.onnx` | Local FP32 export | [IDEA-CCNL/Taiyi-CLIP-Roberta-102M-Chinese at `bc37cdc`][taiyi-clip] |
| `crates/clip/assets/models/taiyi-clip-roberta-102m-vit-base-patch32/image_encoder.onnx` | Local FP32 export | [openai/clip-vit-base-patch32 at `3d74acf`][openai-clip] |
| `crates/waifu-sensor/assets/models/ml-danbooru/ml_caformer_m36_dec-5-97527.onnx` | [Direct ONNX download][ml-danbooru-download] | [deepghs/ml-danbooru-onnx at `eb90583`][ml-danbooru] |

The ML-Danbooru file must have SHA-256
`4ea7aa66df59632c71e036cd9eda9c89e0eaf58bcd39bf9718e63583378cba75`.
Its checksum is also enforced by
[`manifest.json`](../crates/waifu-sensor/assets/models/ml-danbooru/manifest.json).

Source revisions, preprocessing parameters, and license notes for the CLIP
exports are stored beside the models in `model.json` and `NOTICE.md`. The
Chinese-CLIP upstream project did not declare a license at the pinned revision;
do not assume redistribution or commercial-use rights.

[chinese-clip]: https://huggingface.co/OFA-Sys/chinese-clip-vit-base-patch16/tree/36e679e65c2a2fead755ae21162091293ad37834
[taiyi-clip]: https://huggingface.co/IDEA-CCNL/Taiyi-CLIP-Roberta-102M-Chinese/tree/bc37cdc4554e7d30b11659221736cf9d7d35ee33
[openai-clip]: https://huggingface.co/openai/clip-vit-base-patch32/tree/3d74acf9a28c67741b2f4f2ea7635f0aaf6f0268
[gui-clip-text]: https://huggingface.co/zihuv/chinese-clip-vit-base-patch16-onnx/resolve/8001064569c647b9d9ae7756872ba8c334753d91/text.onnx
[gui-clip-image]: https://huggingface.co/zihuv/chinese-clip-vit-base-patch16-onnx/resolve/8001064569c647b9d9ae7756872ba8c334753d91/visual.onnx
[ml-danbooru]: https://huggingface.co/deepghs/ml-danbooru-onnx/tree/eb9058324a741f1b90d4db168f6e1d6b6cb7e63d
[ml-danbooru-download]: https://huggingface.co/deepghs/ml-danbooru-onnx/resolve/eb9058324a741f1b90d4db168f6e1d6b6cb7e63d/ml_caformer_m36_dec-5-97527.onnx
