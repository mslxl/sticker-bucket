#!/usr/bin/env python3
"""Export the two bundled CLIP variants to the crate's FP32 ONNX contract."""

from __future__ import annotations

import argparse
import gc
import json
from dataclasses import dataclass
from pathlib import Path

import onnx
import torch
from transformers import (
    AutoTokenizer,
    BertForSequenceClassification,
    ChineseCLIPImageProcessor,
    ChineseCLIPModel,
    CLIPImageProcessor,
    CLIPModel,
)


@dataclass(frozen=True)
class Source:
    repository: str
    revision: str
    context_length: int


CHINESE_CLIP = Source(
    "OFA-Sys/chinese-clip-vit-base-patch16",
    "36e679e65c2a2fead755ae21162091293ad37834",
    512,
)
TAIYI_TEXT = Source(
    "IDEA-CCNL/Taiyi-CLIP-Roberta-102M-Chinese",
    "bc37cdc4554e7d30b11659221736cf9d7d35ee33",
    512,
)
OPENAI_IMAGE = Source(
    "openai/clip-vit-base-patch32",
    "3d74acf9a28c67741b2f4f2ea7635f0aaf6f0268",
    77,
)


class ChineseClipText(torch.nn.Module):
    def __init__(self, model: ChineseCLIPModel) -> None:
        super().__init__()
        self.model = model

    def forward(
        self,
        input_ids: torch.Tensor,
        attention_mask: torch.Tensor,
        token_type_ids: torch.Tensor,
    ) -> torch.Tensor:
        return self.model.get_text_features(
            input_ids=input_ids,
            attention_mask=attention_mask,
            token_type_ids=token_type_ids,
        )


class ChineseClipImage(torch.nn.Module):
    def __init__(self, model: ChineseCLIPModel) -> None:
        super().__init__()
        self.model = model

    def forward(self, pixel_values: torch.Tensor) -> torch.Tensor:
        return self.model.get_image_features(pixel_values=pixel_values)


class TaiyiText(torch.nn.Module):
    def __init__(self, model: BertForSequenceClassification) -> None:
        super().__init__()
        self.model = model

    def forward(
        self,
        input_ids: torch.Tensor,
        attention_mask: torch.Tensor,
        token_type_ids: torch.Tensor,
    ) -> torch.Tensor:
        return self.model(
            input_ids=input_ids,
            attention_mask=attention_mask,
            token_type_ids=token_type_ids,
            return_dict=True,
        ).logits


class OpenAiClipImage(torch.nn.Module):
    def __init__(self, model: CLIPModel) -> None:
        super().__init__()
        self.model = model

    def forward(self, pixel_values: torch.Tensor) -> torch.Tensor:
        return self.model.get_image_features(pixel_values=pixel_values)


def export_text(model: torch.nn.Module, path: Path, context_length: int) -> None:
    sample = (
        torch.ones((2, min(context_length, 8)), dtype=torch.int64),
        torch.ones((2, min(context_length, 8)), dtype=torch.int64),
        torch.zeros((2, min(context_length, 8)), dtype=torch.int64),
    )
    torch.onnx.export(
        model.eval(),
        sample,
        path,
        input_names=["input_ids", "attention_mask", "token_type_ids"],
        output_names=["embeddings"],
        dynamic_axes={
            "input_ids": {0: "batch", 1: "sequence"},
            "attention_mask": {0: "batch", 1: "sequence"},
            "token_type_ids": {0: "batch", 1: "sequence"},
            "embeddings": {0: "batch"},
        },
        opset_version=17,
        do_constant_folding=True,
        dynamo=False,
    )


def export_image(model: torch.nn.Module, path: Path, image_size: int) -> None:
    sample = torch.zeros((2, 3, image_size, image_size), dtype=torch.float32)
    torch.onnx.export(
        model.eval(),
        sample,
        path,
        input_names=["pixel_values"],
        output_names=["embeddings"],
        dynamic_axes={"pixel_values": {0: "batch"}, "embeddings": {0: "batch"}},
        opset_version=17,
        do_constant_folding=True,
        dynamo=False,
    )


def save_tokenizer(source: Source, directory: Path) -> tuple[str, int, int]:
    tokenizer = AutoTokenizer.from_pretrained(
        source.repository,
        revision=source.revision,
        use_fast=True,
    )
    tokenizer.save_pretrained(directory)
    tokenizer_path = directory / "tokenizer.json"
    if not tokenizer_path.is_file():
        raise RuntimeError(f"fast tokenizer did not produce {tokenizer_path}")
    if tokenizer.pad_token_id is None:
        raise RuntimeError(f"{source.repository} tokenizer has no pad token")
    return tokenizer_path.name, int(tokenizer.pad_token_id), source.context_length


def processor_parameters(
    processor: ChineseCLIPImageProcessor | CLIPImageProcessor,
) -> tuple[int, str, list[float], list[float]]:
    size = processor.size
    if "height" in size and "width" in size:
        if size["height"] != size["width"]:
            raise RuntimeError(f"non-square image processor size {size}")
        image_size = int(size["height"])
    elif "shortest_edge" in size:
        image_size = int(size["shortest_edge"])
    else:
        raise RuntimeError(f"unsupported image processor size {size}")
    resize = "shortest-edge-center-crop" if processor.do_center_crop else "stretch"
    mean = [float(value) for value in processor.image_mean]
    std = [float(value) for value in processor.image_std]
    return image_size, resize, mean, std


def write_manifest(directory: Path, manifest: dict[str, object]) -> None:
    (directory / "model.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def verify_onnx(path: Path, expected_inputs: list[str]) -> None:
    onnx.checker.check_model(str(path), full_check=True)
    model = onnx.load(path, load_external_data=False)
    inputs = [value.name for value in model.graph.input]
    outputs = [value.name for value in model.graph.output]
    if inputs != expected_inputs:
        raise RuntimeError(f"{path} inputs are {inputs}, expected {expected_inputs}")
    if outputs != ["embeddings"]:
        raise RuntimeError(f"{path} outputs are {outputs}, expected ['embeddings']")
    floating_types = {
        onnx.TensorProto.FLOAT,
        onnx.TensorProto.FLOAT16,
        onnx.TensorProto.DOUBLE,
        onnx.TensorProto.BFLOAT16,
    }
    wrong = [
        initializer.name
        for initializer in model.graph.initializer
        if initializer.data_type in floating_types
        and initializer.data_type != onnx.TensorProto.FLOAT
    ]
    if wrong:
        raise RuntimeError(f"{path} contains non-FP32 initializers: {wrong[:10]}")


@torch.inference_mode()
def export_chinese_clip(output_root: Path) -> None:
    directory = output_root / "chinese-clip-vit-base-patch16"
    directory.mkdir(parents=True, exist_ok=True)
    tokenizer_file, pad_token_id, context_length = save_tokenizer(CHINESE_CLIP, directory)
    processor = ChineseCLIPImageProcessor.from_pretrained(
        CHINESE_CLIP.repository,
        revision=CHINESE_CLIP.revision,
    )
    image_size, resize, mean, std = processor_parameters(processor)
    model = ChineseCLIPModel.from_pretrained(
        CHINESE_CLIP.repository,
        revision=CHINESE_CLIP.revision,
        torch_dtype=torch.float32,
        use_safetensors=False,
    ).eval()
    if model.config.projection_dim != 512:
        raise RuntimeError(f"unexpected projection dimension {model.config.projection_dim}")
    export_text(ChineseClipText(model), directory / "text_encoder.onnx", context_length)
    export_image(ChineseClipImage(model), directory / "image_encoder.onnx", image_size)
    write_manifest(
        directory,
        {
            "schema_version": 1,
            "id": "chinese-clip-vit-base-patch16-fp32",
            "source": CHINESE_CLIP.repository,
            "revision": CHINESE_CLIP.revision,
            "license": None,
            "embedding_dimension": 512,
            "text_model": "text_encoder.onnx",
            "image_model": "image_encoder.onnx",
            "tokenizer": tokenizer_file,
            "context_length": context_length,
            "pad_token_id": pad_token_id,
            "image_size": image_size,
            "image_resize": resize,
            "image_mean": mean,
            "image_std": std,
        },
    )
    verify_onnx(
        directory / "text_encoder.onnx",
        ["input_ids", "attention_mask", "token_type_ids"],
    )
    verify_onnx(directory / "image_encoder.onnx", ["pixel_values"])
    del model
    gc.collect()


@torch.inference_mode()
def export_taiyi_clip(output_root: Path) -> None:
    directory = output_root / "taiyi-clip-roberta-102m-vit-base-patch32"
    directory.mkdir(parents=True, exist_ok=True)
    tokenizer_file, pad_token_id, context_length = save_tokenizer(TAIYI_TEXT, directory)
    text_model = BertForSequenceClassification.from_pretrained(
        TAIYI_TEXT.repository,
        revision=TAIYI_TEXT.revision,
        torch_dtype=torch.float32,
        use_safetensors=False,
    ).eval()
    if text_model.config.num_labels != 512:
        raise RuntimeError(f"unexpected Taiyi output dimension {text_model.config.num_labels}")
    export_text(TaiyiText(text_model), directory / "text_encoder.onnx", context_length)
    del text_model
    gc.collect()

    processor = CLIPImageProcessor.from_pretrained(
        OPENAI_IMAGE.repository,
        revision=OPENAI_IMAGE.revision,
    )
    image_size, resize, mean, std = processor_parameters(processor)
    image_model = CLIPModel.from_pretrained(
        OPENAI_IMAGE.repository,
        revision=OPENAI_IMAGE.revision,
        torch_dtype=torch.float32,
        use_safetensors=False,
    ).eval()
    if image_model.config.projection_dim != 512:
        raise RuntimeError(f"unexpected projection dimension {image_model.config.projection_dim}")
    export_image(OpenAiClipImage(image_model), directory / "image_encoder.onnx", image_size)
    write_manifest(
        directory,
        {
            "schema_version": 1,
            "id": "taiyi-clip-roberta-102m-vit-base-patch32-fp32",
            "source": f"{TAIYI_TEXT.repository} + {OPENAI_IMAGE.repository}",
            "revision": f"{TAIYI_TEXT.revision} + {OPENAI_IMAGE.revision}",
            "license": "Apache-2.0 and MIT",
            "embedding_dimension": 512,
            "text_model": "text_encoder.onnx",
            "image_model": "image_encoder.onnx",
            "tokenizer": tokenizer_file,
            "context_length": context_length,
            "pad_token_id": pad_token_id,
            "image_size": image_size,
            "image_resize": resize,
            "image_mean": mean,
            "image_std": std,
        },
    )
    verify_onnx(
        directory / "text_encoder.onnx",
        ["input_ids", "attention_mask", "token_type_ids"],
    )
    verify_onnx(directory / "image_encoder.onnx", ["pixel_values"])
    del image_model
    gc.collect()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "model",
        choices=["all", "chinese-clip", "taiyi-clip"],
        help="model bundle to export",
    )
    parser.add_argument(
        "--output-root",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "assets" / "models",
    )
    args = parser.parse_args()

    torch.set_grad_enabled(False)
    if args.model in ("all", "chinese-clip"):
        export_chinese_clip(args.output_root)
    if args.model in ("all", "taiyi-clip"):
        export_taiyi_clip(args.output_root)


if __name__ == "__main__":
    main()
