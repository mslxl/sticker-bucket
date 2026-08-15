# CLIP model metadata

The GUI downloads the FP32 ONNX files on first launch. This directory keeps the
small manifests and tokenizer metadata needed to validate those downloads.
See the workspace's [ONNX model source index](../../../docs/onnx-model-sources.md)
for pinned upstream snapshot links.

## chinese-clip-vit-base-patch16

Source: `OFA-Sys/chinese-clip-vit-base-patch16`, revision
`36e679e65c2a2fead755ae21162091293ad37834`.

The upstream model card and repository do not declare a license. Its assets are
included for local evaluation only; do not assume that redistribution or
commercial use is permitted without obtaining authorization from the upstream
rightsholder.

## taiyi-clip-roberta-102m-vit-base-patch32

Text source: `IDEA-CCNL/Taiyi-CLIP-Roberta-102M-Chinese` under Apache-2.0.
The paired image encoder comes from `openai/clip-vit-base-patch32`.

Exact source revisions and preprocessing parameters are recorded in each
model's `model.json`.
