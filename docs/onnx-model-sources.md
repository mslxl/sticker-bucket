# ONNX model sources

The ONNX files committed to this repository are stored with Git LFS. After
cloning, download the repository copies with:

```sh
git lfs pull --include='crates/clip/assets/models/**/*.onnx,crates/waifu-sensor/assets/models/**/*.onnx'
```

The table below records where each model originated. The four CLIP files are
local FP32 exports; their upstream projects do not publish these exact ONNX
artifacts. Recreate them from the pinned Hugging Face snapshots with
[`crates/clip/tools/export_models.py`](../crates/clip/tools/export_models.py).

| Repository file | Acquisition | Upstream source |
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
[ml-danbooru]: https://huggingface.co/deepghs/ml-danbooru-onnx/tree/eb9058324a741f1b90d4db168f6e1d6b6cb7e63d
[ml-danbooru-download]: https://huggingface.co/deepghs/ml-danbooru-onnx/resolve/eb9058324a741f1b90d4db168f6e1d6b6cb7e63d/ml_caformer_m36_dec-5-97527.onnx
