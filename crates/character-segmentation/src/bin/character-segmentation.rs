use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use character_segmentation::{
    AnimeInsSegBoundingBox, AnimeInsSegOptions, AnimeInsSegmentation, DualModelSegmenter,
    MergedSegmentation, SegmentationBackend, Yolo11BoundingBox, Yolo11Segmentation,
    Yolo11SegmentationOptions, merge_segmentations,
};
use clap::Parser;
use image::{DynamicImage, GrayImage, Rgb, RgbImage, RgbaImage};
use serde::Serialize;

const ANIME_OUTPUT_DIRECTORY: &str = "animeinsseg";
const YOLO_OUTPUT_DIRECTORY: &str = "yolo11n-seg";
const MERGED_OUTPUT_DIRECTORY: &str = "merged";
const COLORS: [[u8; 3]; 6] = [
    [232, 102, 48],
    [92, 176, 64],
    [43, 137, 225],
    [205, 82, 180],
    [205, 190, 55],
    [96, 77, 210],
];

#[derive(Debug, Parser)]
#[command(about = "Run AnimeInsSeg and YOLO11 ONNX instance segmentation together")]
struct Args {
    /// Path to the raw rtmdetl_e60 AnimeInsSeg ONNX export.
    #[arg(long)]
    anime_model: PathBuf,

    /// Path to yolo11n-seg.onnx or a compatible fine-tuned export.
    #[arg(long)]
    yolo_model: PathBuf,

    /// Anime or photographic image to segment with both models.
    #[arg(long)]
    image: PathBuf,

    /// Directory that receives backend subdirectories and summary.json.
    #[arg(long)]
    output_dir: PathBuf,

    /// Minimum AnimeInsSeg confidence, in [0, 1].
    #[arg(long, default_value_t = 0.6)]
    anime_confidence: f32,

    /// AnimeInsSeg NMS IoU threshold.
    #[arg(long, default_value_t = 0.6)]
    anime_iou: f32,

    /// AnimeInsSeg binary mask probability threshold.
    #[arg(long, default_value_t = 0.5)]
    anime_mask_threshold: f32,

    /// Maximum number of AnimeInsSeg instances.
    #[arg(long, default_value_t = 100)]
    anime_max_instances: usize,

    /// Minimum YOLO11 confidence, in [0, 1].
    #[arg(long, default_value_t = 0.25)]
    yolo_confidence: f32,

    /// YOLO11 same-class NMS IoU threshold.
    #[arg(long, default_value_t = 0.7)]
    yolo_iou: f32,

    /// Cross-model NMS IoU threshold used for the merged output.
    #[arg(long, default_value_t = 0.6)]
    merge_iou: f32,

    /// Keep only this exact YOLO11 class label.
    #[arg(long, default_value = "person")]
    yolo_class_label: String,

    /// Newline-delimited labels for a fine-tuned YOLO11 model.
    #[arg(long)]
    yolo_labels: Option<PathBuf>,

    /// Square resolution used by both ONNX exports.
    #[arg(long, default_value_t = 640)]
    input_size: u32,
}

#[derive(Serialize)]
struct AnimeOutputInstance {
    index: usize,
    confidence: f32,
    bounding_box: AnimeInsSegBoundingBox,
    mask_pixels: u64,
    mask_file: String,
    cutout_file: String,
}

#[derive(Serialize)]
struct YoloOutputInstance {
    index: usize,
    class_id: usize,
    class_name: String,
    confidence: f32,
    bounding_box: Yolo11BoundingBox,
    mask_pixels: u64,
    mask_file: String,
    cutout_file: String,
}

#[derive(Serialize)]
struct MergedOutputInstance {
    index: usize,
    backend: SegmentationBackend,
    class_id: Option<usize>,
    class_name: Option<String>,
    confidence: f32,
    bounding_box: AnimeInsSegBoundingBox,
    mask_pixels: u64,
    mask_file: String,
    cutout_file: String,
}

#[derive(Serialize)]
struct BackendSummary {
    model: String,
    output_directory: String,
    instance_count: usize,
}

#[derive(Serialize)]
struct MergeSummary {
    output_directory: String,
    instance_count: usize,
    iou_threshold: f32,
}

#[derive(Serialize)]
struct RunSummary {
    image: String,
    animeinsseg: BackendSummary,
    yolo11n_seg: BackendSummary,
    merged: MergeSummary,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let anime_options = AnimeInsSegOptions {
        input_size: args.input_size,
        confidence_threshold: args.anime_confidence,
        iou_threshold: args.anime_iou,
        mask_threshold: args.anime_mask_threshold,
        max_instances: args.anime_max_instances,
        ..AnimeInsSegOptions::default()
    };
    let mut yolo_options = Yolo11SegmentationOptions {
        input_size: args.input_size,
        confidence_threshold: args.yolo_confidence,
        iou_threshold: args.yolo_iou,
        ..Yolo11SegmentationOptions::default()
    };
    if let Some(labels_path) = &args.yolo_labels {
        yolo_options.labels = read_labels(labels_path)?;
    }

    let source = image::open(&args.image)
        .with_context(|| format!("failed to open input image {}", args.image.display()))?;
    let mut segmenter = DualModelSegmenter::load(
        &args.anime_model,
        &args.yolo_model,
        anime_options,
        yolo_options,
    )
    .context("failed to load one of the segmentation models")?;
    let results = segmenter
        .segment(&source)
        .context("parallel dual-model segmentation failed")?;
    let yolo_instances = filter_yolo_class(results.yolo11, &args.yolo_class_label)?;
    let merged_instances =
        merge_segmentations(&results.animeinsseg, &yolo_instances, args.merge_iou)
            .context("cross-model NMS failed")?;

    fs::create_dir_all(&args.output_dir).with_context(|| {
        format!(
            "failed to create output directory {}",
            args.output_dir.display()
        )
    })?;
    let anime_output_dir = args.output_dir.join(ANIME_OUTPUT_DIRECTORY);
    let yolo_output_dir = args.output_dir.join(YOLO_OUTPUT_DIRECTORY);
    let merged_output_dir = args.output_dir.join(MERGED_OUTPUT_DIRECTORY);
    let anime_manifest = write_anime_results(&results.animeinsseg, &source, &anime_output_dir)?;
    let yolo_manifest = write_yolo_results(&yolo_instances, &source, &yolo_output_dir)?;
    let merged_manifest = write_merged_results(&merged_instances, &source, &merged_output_dir)?;

    let summary = RunSummary {
        image: args.image.display().to_string(),
        animeinsseg: BackendSummary {
            model: args.anime_model.display().to_string(),
            output_directory: ANIME_OUTPUT_DIRECTORY.to_owned(),
            instance_count: anime_manifest.len(),
        },
        yolo11n_seg: BackendSummary {
            model: args.yolo_model.display().to_string(),
            output_directory: YOLO_OUTPUT_DIRECTORY.to_owned(),
            instance_count: yolo_manifest.len(),
        },
        merged: MergeSummary {
            output_directory: MERGED_OUTPUT_DIRECTORY.to_owned(),
            instance_count: merged_manifest.len(),
            iou_threshold: args.merge_iou,
        },
    };
    let summary_path = args.output_dir.join("summary.json");
    fs::write(&summary_path, serde_json::to_vec_pretty(&summary)?)
        .with_context(|| format!("failed to write {}", summary_path.display()))?;
    println!(
        "wrote {} AnimeInsSeg, {} YOLO11, and {} merged instance(s) to {}",
        anime_manifest.len(),
        yolo_manifest.len(),
        merged_manifest.len(),
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

fn filter_yolo_class(
    instances: Vec<Yolo11Segmentation>,
    class_label: &str,
) -> Result<Vec<Yolo11Segmentation>> {
    if class_label.trim().is_empty() {
        bail!("--yolo-class-label must not be empty");
    }
    Ok(instances
        .into_iter()
        .filter(|instance| instance.class_name == class_label)
        .collect())
}

fn write_anime_results(
    instances: &[AnimeInsSegmentation],
    source: &DynamicImage,
    output_dir: &Path,
) -> Result<Vec<AnimeOutputInstance>> {
    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;
    let mut overlay = source.to_rgb8();
    let manifest = instances
        .iter()
        .enumerate()
        .map(|(index, instance)| {
            blend_instance(
                &mut overlay,
                &instance.mask,
                instance.bounding_box.into(),
                COLORS[index % COLORS.len()],
            );
            let mask_file = format!("instance-{index:03}-mask.png");
            let cutout_file = format!("instance-{index:03}.png");
            save_mask_and_cutout(&instance.mask, source, output_dir, &mask_file, &cutout_file)?;
            Ok(AnimeOutputInstance {
                index,
                confidence: instance.confidence,
                bounding_box: instance.bounding_box,
                mask_pixels: mask_pixels(&instance.mask),
                mask_file,
                cutout_file,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    write_backend_files(output_dir, &overlay, &manifest)?;
    Ok(manifest)
}

fn write_yolo_results(
    instances: &[Yolo11Segmentation],
    source: &DynamicImage,
    output_dir: &Path,
) -> Result<Vec<YoloOutputInstance>> {
    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;
    let mut overlay = source.to_rgb8();
    let manifest = instances
        .iter()
        .enumerate()
        .map(|(index, instance)| {
            blend_instance(
                &mut overlay,
                &instance.mask,
                instance.bounding_box.into(),
                COLORS[index % COLORS.len()],
            );
            let stem = format!(
                "instance-{index:03}-{}",
                file_safe_label(&instance.class_name)
            );
            let mask_file = format!("{stem}-mask.png");
            let cutout_file = format!("{stem}.png");
            save_mask_and_cutout(&instance.mask, source, output_dir, &mask_file, &cutout_file)?;
            Ok(YoloOutputInstance {
                index,
                class_id: instance.class_id,
                class_name: instance.class_name.clone(),
                confidence: instance.confidence,
                bounding_box: instance.bounding_box,
                mask_pixels: mask_pixels(&instance.mask),
                mask_file,
                cutout_file,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    write_backend_files(output_dir, &overlay, &manifest)?;
    Ok(manifest)
}

fn write_merged_results(
    instances: &[MergedSegmentation],
    source: &DynamicImage,
    output_dir: &Path,
) -> Result<Vec<MergedOutputInstance>> {
    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;
    let mut overlay = source.to_rgb8();
    let manifest = instances
        .iter()
        .enumerate()
        .map(|(index, instance)| {
            blend_instance(
                &mut overlay,
                &instance.mask,
                instance.bounding_box.into(),
                COLORS[index % COLORS.len()],
            );
            let backend_label = match instance.backend {
                SegmentationBackend::AnimeInsSeg => "animeinsseg".to_owned(),
                SegmentationBackend::Yolo11 => {
                    let class_name = instance.class_name.as_deref().ok_or_else(|| {
                        anyhow::anyhow!("merged YOLO11 instance is missing its class name")
                    })?;
                    format!("yolo11-{}", file_safe_label(class_name))
                }
            };
            let stem = format!("instance-{index:03}-{backend_label}");
            let mask_file = format!("{stem}-mask.png");
            let cutout_file = format!("{stem}.png");
            save_mask_and_cutout(&instance.mask, source, output_dir, &mask_file, &cutout_file)?;
            Ok(MergedOutputInstance {
                index,
                backend: instance.backend,
                class_id: instance.class_id,
                class_name: instance.class_name.clone(),
                confidence: instance.confidence,
                bounding_box: instance.bounding_box,
                mask_pixels: mask_pixels(&instance.mask),
                mask_file,
                cutout_file,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    write_backend_files(output_dir, &overlay, &manifest)?;
    Ok(manifest)
}

fn save_mask_and_cutout(
    mask: &GrayImage,
    source: &DynamicImage,
    output_dir: &Path,
    mask_file: &str,
    cutout_file: &str,
) -> Result<()> {
    let mask_path = output_dir.join(mask_file);
    mask.save(&mask_path)
        .with_context(|| format!("failed to write {}", mask_path.display()))?;
    let cutout_path = output_dir.join(cutout_file);
    transparent_cutout(source, mask)
        .save(&cutout_path)
        .with_context(|| format!("failed to write {}", cutout_path.display()))?;
    Ok(())
}

fn write_backend_files<T: Serialize>(
    output_dir: &Path,
    overlay: &RgbImage,
    manifest: &[T],
) -> Result<()> {
    let overlay_path = output_dir.join("overlay.png");
    overlay
        .save(&overlay_path)
        .with_context(|| format!("failed to write {}", overlay_path.display()))?;
    let manifest_path = output_dir.join("instances.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(manifest)?)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;
    Ok(())
}

fn transparent_cutout(source: &DynamicImage, mask: &GrayImage) -> RgbaImage {
    let mut output = source.to_rgba8();
    for (pixel, mask_pixel) in output.pixels_mut().zip(mask.pixels()) {
        pixel[3] = ((u16::from(pixel[3]) * u16::from(mask_pixel[0])) / 255) as u8;
    }
    output
}

fn mask_pixels(mask: &GrayImage) -> u64 {
    mask.pixels().filter(|pixel| pixel[0] != 0).count() as u64
}

fn blend_instance(
    overlay: &mut RgbImage,
    mask: &GrayImage,
    bounding_box: BoxCoordinates,
    color: [u8; 3],
) {
    for (pixel, mask_pixel) in overlay.pixels_mut().zip(mask.pixels()) {
        if mask_pixel[0] != 0 {
            for channel in 0..3 {
                pixel[channel] =
                    ((u16::from(pixel[channel]) + u16::from(color[channel])) / 2) as u8;
            }
        }
    }
    draw_box(overlay, bounding_box, color);
}

#[derive(Clone, Copy)]
struct BoxCoordinates {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl From<AnimeInsSegBoundingBox> for BoxCoordinates {
    fn from(value: AnimeInsSegBoundingBox) -> Self {
        Self {
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

impl From<Yolo11BoundingBox> for BoxCoordinates {
    fn from(value: Yolo11BoundingBox) -> Self {
        Self {
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

fn draw_box(image: &mut RgbImage, bounding_box: BoxCoordinates, color: [u8; 3]) {
    let max_x = image.width().saturating_sub(1);
    let max_y = image.height().saturating_sub(1);
    let left = bounding_box.x.round().clamp(0.0, max_x as f32) as u32;
    let top = bounding_box.y.round().clamp(0.0, max_y as f32) as u32;
    let right = (bounding_box.x + bounding_box.width)
        .round()
        .clamp(left as f32, max_x as f32) as u32;
    let bottom = (bounding_box.y + bounding_box.height)
        .round()
        .clamp(top as f32, max_y as f32) as u32;
    let color = Rgb(color);
    for thickness in 0..2_u32 {
        let x0 = left.saturating_add(thickness).min(right);
        let x1 = right.saturating_sub(thickness).max(x0);
        let y0 = top.saturating_add(thickness).min(bottom);
        let y1 = bottom.saturating_sub(thickness).max(y0);
        for x in x0..=x1 {
            image.put_pixel(x, y0, color);
            image.put_pixel(x, y1, color);
        }
        for y in y0..=y1 {
            image.put_pixel(x0, y, color);
            image.put_pixel(x1, y, color);
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use image::Luma;

    fn yolo_instance(class_id: usize, class_name: &str) -> Yolo11Segmentation {
        Yolo11Segmentation {
            class_id,
            class_name: class_name.to_owned(),
            confidence: 0.75,
            bounding_box: Yolo11BoundingBox {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            },
            mask: GrayImage::from_pixel(2, 2, Luma([255])),
        }
    }

    #[test]
    fn yolo_filter_retains_only_the_requested_character_class() {
        let filtered = filter_yolo_class(
            vec![yolo_instance(0, "person"), yolo_instance(15, "cat")],
            "person",
        )
        .unwrap();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].class_id, 0);
        assert_eq!(filtered[0].class_name, "person");
        assert_eq!(filtered[0].mask.get_pixel(0, 0), &Luma([255]));
    }

    #[test]
    fn file_safe_label_replaces_path_punctuation() {
        assert_eq!(file_safe_label("anime/person 1"), "anime-person-1");
    }
}
