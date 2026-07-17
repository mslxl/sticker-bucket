use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use character_segmentation::yolo11::{BoundingBox, Segmentation, SegmentationOptions, Yolo11Seg};
use clap::Parser;
use image::{DynamicImage, RgbaImage};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(about = "Run a local YOLO11 ONNX segmentation model")]
struct Args {
    /// Path to yolo11n-seg.onnx (or a compatible fine-tuned ONNX export).
    #[arg(long)]
    model: PathBuf,

    /// Image to segment.
    #[arg(long)]
    image: PathBuf,

    /// Directory that receives transparent instance PNGs and instances.json.
    #[arg(long)]
    output_dir: PathBuf,

    /// Minimum confidence, in [0, 1].
    #[arg(long, default_value_t = 0.25)]
    confidence: f32,

    /// Suppress same-class boxes whose IoU exceeds this value.
    #[arg(long, default_value_t = 0.7)]
    iou: f32,

    /// Square ONNX export resolution, such as 640.
    #[arg(long, default_value_t = 640)]
    input_size: u32,

    /// Keep only this exact class label, such as `person`.
    #[arg(long)]
    class_label: Option<String>,

    /// Newline-delimited class names for a fine-tuned model.
    #[arg(long)]
    labels: Option<PathBuf>,
}

#[derive(Serialize)]
struct OutputInstance {
    index: usize,
    class_id: usize,
    class_name: String,
    confidence: f32,
    bounding_box: BoundingBox,
    mask_file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut options = SegmentationOptions {
        confidence_threshold: args.confidence,
        iou_threshold: args.iou,
        input_size: args.input_size,
        ..SegmentationOptions::default()
    };
    if let Some(labels_path) = args.labels {
        options.labels = read_labels(&labels_path)?;
    }
    let source = image::open(&args.image)
        .with_context(|| format!("failed to open input image {}", args.image.display()))?;
    let mut model = Yolo11Seg::load(&args.model, options)
        .with_context(|| format!("failed to load ONNX model {}", args.model.display()))?;
    let instances = model
        .segment(&source)
        .context("YOLO11 segmentation failed")?;
    let instances = filter_class(instances, args.class_label.as_deref())?;

    fs::create_dir_all(&args.output_dir).with_context(|| {
        format!(
            "failed to create output directory {}",
            args.output_dir.display()
        )
    })?;
    let manifest = instances
        .iter()
        .enumerate()
        .map(|(index, instance)| write_instance(index, instance, &source, &args.output_dir))
        .collect::<Result<Vec<_>>>()?;
    let manifest_path = args.output_dir.join("instances.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;
    println!(
        "wrote {} instance(s) to {}",
        manifest.len(),
        args.output_dir.display()
    );
    Ok(())
}

fn read_labels(path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read labels from {}", path.display()))?;
    let labels = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if labels.is_empty() {
        bail!("label file {} contains no labels", path.display());
    }
    Ok(labels)
}

fn filter_class(
    instances: Vec<Segmentation>,
    class_label: Option<&str>,
) -> Result<Vec<Segmentation>> {
    let Some(class_label) = class_label else {
        return Ok(instances);
    };
    if class_label.trim().is_empty() {
        bail!("--class-label must not be empty");
    }
    Ok(instances
        .into_iter()
        .filter(|instance| instance.class_name == class_label)
        .collect())
}

fn write_instance(
    index: usize,
    instance: &Segmentation,
    source: &DynamicImage,
    output_dir: &Path,
) -> Result<OutputInstance> {
    let stem = format!(
        "instance-{index:03}-{}",
        file_safe_label(&instance.class_name)
    );
    let mask_file = format!("{stem}.png");
    let path = output_dir.join(&mask_file);
    transparent_cutout(source, &instance.mask)
        .save(&path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(OutputInstance {
        index,
        class_id: instance.class_id,
        class_name: instance.class_name.clone(),
        confidence: instance.confidence,
        bounding_box: instance.bounding_box,
        mask_file,
    })
}

fn transparent_cutout(source: &DynamicImage, mask: &image::GrayImage) -> RgbaImage {
    let mut output = source.to_rgba8();
    for (pixel, mask_pixel) in output.pixels_mut().zip(mask.pixels()) {
        pixel[3] = ((u16::from(pixel[3]) * u16::from(mask_pixel[0])) / 255) as u8;
    }
    output
}

fn file_safe_label(label: &str) -> String {
    label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect()
}
