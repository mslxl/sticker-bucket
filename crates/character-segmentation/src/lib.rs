#![forbid(unsafe_code)]

//! Local character instance segmentation using AnimeInsSeg and YOLO11 ONNX models.
//!
//! [`DualModelSegmenter`] runs both model sessions concurrently and returns
//! their independent results. The AnimeInsSeg backend implements the
//! MMDetection post-processing path: point prior generation, distance-box
//! decoding, per-level top-k filtering, NMS, and the three-layer dynamic mask
//! head.

pub mod yolo11;

use std::path::Path;

use image::{DynamicImage, GrayImage, ImageBuffer, Luma, Rgb, RgbImage};
use ort::{session::Session, value::Tensor};
use serde::Serialize;
use thiserror::Error;

pub use image;
pub use yolo11::{
    BoundingBox as Yolo11BoundingBox, Segmentation as Yolo11Segmentation,
    SegmentationOptions as Yolo11SegmentationOptions, Yolo11Seg,
};

const DEFAULT_INPUT_SIZE: u32 = 640;
const STRIDES: [usize; 3] = [8, 16, 32];
const MASK_FEATURE_CHANNELS: usize = 8;
const DYNAMIC_KERNEL_PARAMETERS: usize = 169;
const DYNAMIC_CHANNELS: usize = 8;
const FIRST_LAYER_WEIGHTS: usize = 80;
const SECOND_LAYER_WEIGHTS: usize = 64;
const THIRD_LAYER_WEIGHTS: usize = 8;
const FIRST_LAYER_BIAS_OFFSET: usize = 152;
const SECOND_LAYER_BIAS_OFFSET: usize = 160;
const THIRD_LAYER_BIAS_OFFSET: usize = 168;
const CROSS_MODEL_MASK_IOU_THRESHOLD: f32 = 0.5;

/// Errors returned by the segmentation APIs.
#[derive(Debug, Error)]
pub enum Error {
    #[error("ONNX Runtime operation failed: {0}")]
    Onnx(#[from] ort::Error),

    #[error("invalid segmentation configuration: {0}")]
    InvalidConfiguration(String),

    #[error("model output is incompatible with AnimeInsSeg: {0}")]
    InvalidModelOutput(String),

    #[error("YOLO11 segmentation failed: {0}")]
    Yolo11(#[from] yolo11::Error),

    #[error("{backend} inference worker panicked")]
    WorkerPanicked { backend: &'static str },
}

pub type Result<T> = std::result::Result<T, Error>;

/// Independent instance-segmentation results from both model backends.
#[derive(Debug)]
pub struct DualModelSegmentations {
    pub animeinsseg: Vec<AnimeInsSegmentation>,
    pub yolo11: Vec<Yolo11Segmentation>,
}

/// Model backend that produced a merged segmentation instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SegmentationBackend {
    AnimeInsSeg,
    Yolo11,
}

/// A segmentation instance retained after cross-model non-maximum suppression.
#[derive(Debug)]
pub struct MergedSegmentation {
    pub backend: SegmentationBackend,
    pub class_id: Option<usize>,
    pub class_name: Option<String>,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
    pub mask: GrayImage,
}

/// Merges AnimeInsSeg and YOLO11 results using confidence-ordered box and mask NMS.
///
/// Callers should filter YOLO11 results to the desired character class before
/// invoking this function. A candidate is suppressed when either its bounding-box
/// IoU exceeds `iou_threshold` or its binary mask IoU exceeds 0.5. Raw backend
/// results are not modified.
pub fn merge_segmentations(
    anime: &[AnimeInsSegmentation],
    yolo: &[Yolo11Segmentation],
    iou_threshold: f32,
) -> Result<Vec<MergedSegmentation>> {
    if !iou_threshold.is_finite() || !(0.0..=1.0).contains(&iou_threshold) {
        return Err(Error::InvalidConfiguration(format!(
            "cross-model NMS iou_threshold must be finite and in [0, 1], got {iou_threshold}"
        )));
    }

    let mut candidates = Vec::with_capacity(anime.len() + yolo.len());
    candidates.extend(anime.iter().map(|instance| MergedSegmentation {
        backend: SegmentationBackend::AnimeInsSeg,
        class_id: None,
        class_name: None,
        confidence: instance.confidence,
        bounding_box: instance.bounding_box,
        mask: instance.mask.clone(),
    }));
    candidates.extend(yolo.iter().map(|instance| MergedSegmentation {
        backend: SegmentationBackend::Yolo11,
        class_id: Some(instance.class_id),
        class_name: Some(instance.class_name.clone()),
        confidence: instance.confidence,
        bounding_box: BoundingBox {
            x: instance.bounding_box.x,
            y: instance.bounding_box.y,
            width: instance.bounding_box.width,
            height: instance.bounding_box.height,
        },
        mask: instance.mask.clone(),
    }));
    candidates.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));

    let mut selected: Vec<MergedSegmentation> = Vec::new();
    'candidate: for candidate in candidates {
        for existing in &selected {
            let bounding_box_iou = candidate
                .bounding_box
                .intersection_over_union(existing.bounding_box);
            if bounding_box_iou > iou_threshold
                || (bounding_box_iou > 0.0
                    && binary_mask_intersection_over_union(&candidate.mask, &existing.mask)?
                        > CROSS_MODEL_MASK_IOU_THRESHOLD)
            {
                continue 'candidate;
            }
        }
        selected.push(candidate);
    }
    Ok(selected)
}

fn binary_mask_intersection_over_union(left: &GrayImage, right: &GrayImage) -> Result<f32> {
    if left.dimensions() != right.dimensions() {
        return Err(Error::InvalidConfiguration(format!(
            "cannot compare masks with different dimensions: {}x{} and {}x{}",
            left.width(),
            left.height(),
            right.width(),
            right.height()
        )));
    }

    let mut intersection = 0_u64;
    let mut union = 0_u64;
    for (left_pixel, right_pixel) in left.pixels().zip(right.pixels()) {
        let left_foreground = left_pixel[0] > 0;
        let right_foreground = right_pixel[0] > 0;
        intersection += u64::from(left_foreground && right_foreground);
        union += u64::from(left_foreground || right_foreground);
    }
    if union == 0 {
        Ok(0.0)
    } else {
        Ok(intersection as f32 / union as f32)
    }
}

/// AnimeInsSeg and YOLO11 sessions that run concurrently for each image.
pub struct DualModelSegmenter {
    animeinsseg: AnimeInsSeg,
    yolo11: Yolo11Seg,
}

impl DualModelSegmenter {
    /// Loads both ONNX models with backend-specific options.
    pub fn load(
        animeinsseg_model_path: impl AsRef<Path>,
        yolo11_model_path: impl AsRef<Path>,
        animeinsseg_options: AnimeInsSegOptions,
        yolo11_options: Yolo11SegmentationOptions,
    ) -> Result<Self> {
        let animeinsseg = AnimeInsSeg::load(animeinsseg_model_path, animeinsseg_options)?;
        let yolo11 = Yolo11Seg::load(yolo11_model_path, yolo11_options)?;
        Ok(Self {
            animeinsseg,
            yolo11,
        })
    }

    /// Runs AnimeInsSeg and YOLO11 concurrently on the same image.
    pub fn segment(&mut self, image: &DynamicImage) -> Result<DualModelSegmentations> {
        let Self {
            animeinsseg,
            yolo11,
        } = self;
        let (animeinsseg_result, yolo11_result) = std::thread::scope(|scope| {
            let animeinsseg_worker = scope.spawn(|| animeinsseg.segment(image));
            let yolo11_worker = scope.spawn(|| yolo11.segment(image));
            (animeinsseg_worker.join(), yolo11_worker.join())
        });
        let animeinsseg = animeinsseg_result.map_err(|_| Error::WorkerPanicked {
            backend: "AnimeInsSeg",
        })??;
        let yolo11 = yolo11_result.map_err(|_| Error::WorkerPanicked { backend: "YOLO11" })??;
        Ok(DualModelSegmentations {
            animeinsseg,
            yolo11,
        })
    }
}

/// A rectangle in original-image pixel coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Explicit backend-qualified name for an AnimeInsSeg bounding box.
pub type AnimeInsSegBoundingBox = BoundingBox;

impl BoundingBox {
    fn intersection_over_union(self, other: Self) -> f32 {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);
        let intersection_width = (right - left).max(0.0);
        let intersection_height = (bottom - top).max(0.0);
        let intersection = intersection_width * intersection_height;
        let union = self.width * self.height + other.width * other.height - intersection;
        if union <= 0.0 {
            0.0
        } else {
            intersection / union
        }
    }
}

/// A detected anime subject and its full-resolution binary instance mask.
#[derive(Debug)]
pub struct Segmentation {
    pub confidence: f32,
    pub bounding_box: BoundingBox,
    pub mask: GrayImage,
}

/// Explicit backend-qualified name for an AnimeInsSeg result.
pub type AnimeInsSegmentation = Segmentation;

/// Inference and MMDetection post-processing parameters.
#[derive(Clone, Debug)]
pub struct SegmentationOptions {
    /// Square resolution used by the ONNX export. The provided export uses 640.
    pub input_size: u32,
    pub confidence_threshold: f32,
    pub iou_threshold: f32,
    pub mask_threshold: f32,
    /// Maximum candidates retained independently from each feature level.
    pub nms_pre: usize,
    pub max_instances: usize,
}

/// Explicit backend-qualified name for AnimeInsSeg inference options.
pub type AnimeInsSegOptions = SegmentationOptions;

impl Default for SegmentationOptions {
    fn default() -> Self {
        Self {
            input_size: DEFAULT_INPUT_SIZE,
            confidence_threshold: 0.6,
            iou_threshold: 0.6,
            mask_threshold: 0.5,
            nms_pre: 1_000,
            max_instances: 100,
        }
    }
}

impl SegmentationOptions {
    fn validate(&self) -> Result<()> {
        if self.input_size == 0 || !self.input_size.is_multiple_of(32) {
            return Err(Error::InvalidConfiguration(format!(
                "input_size must be positive and divisible by 32, got {}",
                self.input_size
            )));
        }
        for (name, value) in [
            ("confidence_threshold", self.confidence_threshold),
            ("iou_threshold", self.iou_threshold),
            ("mask_threshold", self.mask_threshold),
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(Error::InvalidConfiguration(format!(
                    "{name} must be finite and in [0, 1], got {value}"
                )));
            }
        }
        if self.nms_pre == 0 {
            return Err(Error::InvalidConfiguration(
                "nms_pre must be positive".to_owned(),
            ));
        }
        if self.max_instances == 0 {
            return Err(Error::InvalidConfiguration(
                "max_instances must be positive".to_owned(),
            ));
        }
        Ok(())
    }
}

/// A loaded raw AnimeInsSeg ONNX model.
pub struct AnimeInsSeg {
    session: Session,
    options: SegmentationOptions,
}

impl AnimeInsSeg {
    /// Loads an ONNX model exported with the raw AnimeInsSeg output contract.
    pub fn load(model_path: impl AsRef<Path>, options: SegmentationOptions) -> Result<Self> {
        options.validate()?;
        let session = Session::builder()?.commit_from_file(model_path)?;
        Ok(Self { session, options })
    }

    /// Segments every anime subject detected in `image`.
    pub fn segment(&mut self, image: &DynamicImage) -> Result<Vec<Segmentation>> {
        let letterbox = Letterbox::new(image, self.options.input_size)?;
        let size = self.options.input_size as usize;
        let tensor =
            Tensor::from_array(([1_usize, 3, size, size], bgr_to_chw_0_255(&letterbox.image)))?;
        let outputs = self.session.run(ort::inputs![tensor])?;
        let raw = RawOutputs::from_session_outputs(&outputs, size)?;
        let mut candidates = raw.decode_candidates(&letterbox, &self.options)?;
        candidates = non_maximum_suppression(candidates, self.options.iou_threshold);
        candidates.truncate(self.options.max_instances);

        candidates
            .into_iter()
            .map(|candidate| {
                let mask = render_mask(
                    &candidate,
                    &raw.mask_features,
                    &letterbox,
                    self.options.mask_threshold,
                )?;
                Ok(Segmentation {
                    confidence: candidate.confidence,
                    bounding_box: candidate.bounding_box,
                    mask,
                })
            })
            .collect()
    }
}

#[derive(Debug)]
struct Letterbox {
    image: RgbImage,
    original_width: u32,
    original_height: u32,
    scale_x: f32,
    scale_y: f32,
}

impl Letterbox {
    fn new(image: &DynamicImage, input_size: u32) -> Result<Self> {
        if image.width() == 0 || image.height() == 0 {
            return Err(Error::InvalidConfiguration(
                "input image dimensions must be non-zero".to_owned(),
            ));
        }
        let original_width = image.width();
        let original_height = image.height();
        let scale = (input_size as f32 / original_width as f32)
            .min(input_size as f32 / original_height as f32);
        let resized_width = (original_width as f32 * scale).round() as u32;
        let resized_height = (original_height as f32 * scale).round() as u32;
        if resized_width == 0 || resized_height == 0 {
            return Err(Error::InvalidConfiguration(format!(
                "image {}x{} becomes empty at input size {input_size}",
                original_width, original_height
            )));
        }
        let resized = resize_rgb_bilinear(&image.to_rgb8(), resized_width, resized_height);
        let mut output = ImageBuffer::from_pixel(input_size, input_size, Rgb([114, 114, 114]));
        image::imageops::overlay(&mut output, &resized, 0, 0);
        Ok(Self {
            image: output,
            original_width,
            original_height,
            scale_x: resized_width as f32 / original_width as f32,
            scale_y: resized_height as f32 / original_height as f32,
        })
    }

    fn to_original_box(&self, model_box: [f32; 4]) -> BoundingBox {
        let left = (model_box[0] / self.scale_x).clamp(0.0, self.original_width as f32);
        let top = (model_box[1] / self.scale_y).clamp(0.0, self.original_height as f32);
        let right = (model_box[2] / self.scale_x).clamp(left, self.original_width as f32);
        let bottom = (model_box[3] / self.scale_y).clamp(top, self.original_height as f32);
        BoundingBox {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }
}

#[derive(Debug)]
struct ModelTensor {
    shape: Vec<usize>,
    values: Vec<f32>,
}

impl ModelTensor {
    fn named(outputs: &ort::session::SessionOutputs<'_>, name: &str) -> Result<Self> {
        let value = outputs
            .get(name)
            .ok_or_else(|| Error::InvalidModelOutput(format!("missing output named {name}")))?;
        let (shape, values) = value.try_extract_tensor::<f32>()?;
        let shape = shape
            .iter()
            .map(|dimension| {
                usize::try_from(*dimension).map_err(|_| {
                    Error::InvalidModelOutput(format!(
                        "output {name} has invalid dimension {dimension}"
                    ))
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let expected_values = shape.iter().product::<usize>();
        if expected_values != values.len() {
            return Err(Error::InvalidModelOutput(format!(
                "output {name} shape {shape:?} requires {expected_values} values, got {}",
                values.len()
            )));
        }
        Ok(Self {
            shape,
            values: values.to_vec(),
        })
    }

    fn validate_nchw(
        &self,
        name: &str,
        channels: usize,
        height: usize,
        width: usize,
    ) -> Result<()> {
        let expected = [1, channels, height, width];
        if self.shape != expected {
            return Err(Error::InvalidModelOutput(format!(
                "output {name} has shape {:?}, expected {expected:?}",
                self.shape
            )));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct LevelOutputs {
    cls: ModelTensor,
    bbox: ModelTensor,
    kernel: ModelTensor,
    stride: usize,
    height: usize,
    width: usize,
}

#[derive(Debug)]
struct RawOutputs {
    levels: Vec<LevelOutputs>,
    mask_features: ModelTensor,
}

impl RawOutputs {
    fn from_session_outputs(
        outputs: &ort::session::SessionOutputs<'_>,
        input_size: usize,
    ) -> Result<Self> {
        let mut levels = Vec::with_capacity(STRIDES.len());
        for (index, stride) in STRIDES.into_iter().enumerate() {
            let height = input_size / stride;
            let width = input_size / stride;
            let cls_name = format!("cls_s{stride}");
            let bbox_name = format!("bbox_s{stride}");
            let kernel_name = format!("kernel_s{stride}");
            let cls = ModelTensor::named(outputs, &cls_name)?;
            let bbox = ModelTensor::named(outputs, &bbox_name)?;
            let kernel = ModelTensor::named(outputs, &kernel_name)?;
            cls.validate_nchw(&cls_name, 1, height, width)?;
            bbox.validate_nchw(&bbox_name, 4, height, width)?;
            kernel.validate_nchw(&kernel_name, DYNAMIC_KERNEL_PARAMETERS, height, width)?;
            levels.push(LevelOutputs {
                cls,
                bbox,
                kernel,
                stride: STRIDES[index],
                height,
                width,
            });
        }
        let mask_features = ModelTensor::named(outputs, "mask_feat_s8")?;
        let mask_height = input_size / STRIDES[0];
        let mask_width = input_size / STRIDES[0];
        mask_features.validate_nchw(
            "mask_feat_s8",
            MASK_FEATURE_CHANNELS,
            mask_height,
            mask_width,
        )?;
        Ok(Self {
            levels,
            mask_features,
        })
    }

    fn decode_candidates(
        &self,
        letterbox: &Letterbox,
        options: &SegmentationOptions,
    ) -> Result<Vec<Candidate>> {
        let mut candidates = Vec::new();
        for level in &self.levels {
            let mut level_candidates =
                decode_level(level, letterbox, options.confidence_threshold)?;
            level_candidates.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));
            level_candidates.truncate(options.nms_pre);
            candidates.extend(level_candidates);
        }
        Ok(candidates)
    }
}

#[derive(Debug)]
struct Candidate {
    confidence: f32,
    bounding_box: BoundingBox,
    prior_x: f32,
    prior_y: f32,
    prior_stride: f32,
    kernel: Vec<f32>,
}

fn decode_level(
    level: &LevelOutputs,
    letterbox: &Letterbox,
    confidence_threshold: f32,
) -> Result<Vec<Candidate>> {
    let plane = level.height * level.width;
    let mut candidates = Vec::new();
    for y in 0..level.height {
        for x in 0..level.width {
            let position = y * level.width + x;
            let score_logit = level.cls.values[position];
            if !score_logit.is_finite() {
                return Err(Error::InvalidModelOutput(format!(
                    "non-finite class logit at stride {} position ({x}, {y})",
                    level.stride
                )));
            }
            let confidence = sigmoid(score_logit);
            if confidence < confidence_threshold {
                continue;
            }

            let mut distances = [0.0_f32; 4];
            for (channel, distance) in distances.iter_mut().enumerate() {
                *distance = level.bbox.values[channel * plane + position];
            }
            if distances.iter().any(|value| !value.is_finite()) {
                return Err(Error::InvalidModelOutput(format!(
                    "non-finite bbox at stride {} position ({x}, {y})",
                    level.stride
                )));
            }
            let prior_x = x as f32 * level.stride as f32;
            let prior_y = y as f32 * level.stride as f32;
            let input_width = letterbox.image.width() as f32;
            let input_height = letterbox.image.height() as f32;
            let model_box = [
                (prior_x - distances[0]).clamp(0.0, input_width),
                (prior_y - distances[1]).clamp(0.0, input_height),
                (prior_x + distances[2]).clamp(0.0, input_width),
                (prior_y + distances[3]).clamp(0.0, input_height),
            ];
            let bounding_box = letterbox.to_original_box(model_box);
            if bounding_box.width <= 0.0 || bounding_box.height <= 0.0 {
                continue;
            }

            let kernel = (0..DYNAMIC_KERNEL_PARAMETERS)
                .map(|channel| level.kernel.values[channel * plane + position])
                .collect::<Vec<_>>();
            if kernel.iter().any(|value| !value.is_finite()) {
                return Err(Error::InvalidModelOutput(format!(
                    "non-finite dynamic kernel at stride {} position ({x}, {y})",
                    level.stride
                )));
            }
            candidates.push(Candidate {
                confidence,
                bounding_box,
                prior_x,
                prior_y,
                prior_stride: level.stride as f32,
                kernel,
            });
        }
    }
    Ok(candidates)
}

fn non_maximum_suppression(mut candidates: Vec<Candidate>, iou_threshold: f32) -> Vec<Candidate> {
    candidates.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));
    let mut selected: Vec<Candidate> = Vec::new();
    'candidate: for candidate in candidates {
        for existing in &selected {
            if candidate
                .bounding_box
                .intersection_over_union(existing.bounding_box)
                > iou_threshold
            {
                continue 'candidate;
            }
        }
        selected.push(candidate);
    }
    selected
}

fn render_mask(
    candidate: &Candidate,
    mask_features: &ModelTensor,
    letterbox: &Letterbox,
    mask_threshold: f32,
) -> Result<GrayImage> {
    let mask_height = mask_features.shape[2];
    let mask_width = mask_features.shape[3];
    let low_resolution = dynamic_mask_logits(candidate, mask_features)?;
    let input_size = letterbox.image.width() as usize;
    let model_resolution = resize_bilinear(
        &low_resolution,
        mask_width,
        mask_height,
        input_size,
        input_size,
    )?;
    let target_width = (input_size as f32 / letterbox.scale_x).ceil() as usize;
    let target_height = (input_size as f32 / letterbox.scale_y).ceil() as usize;
    if target_height < letterbox.original_height as usize
        || target_width < letterbox.original_width as usize
    {
        return Err(Error::InvalidModelOutput(format!(
            "rescaled mask {target_width}x{target_height} is smaller than original {}x{}",
            letterbox.original_width, letterbox.original_height
        )));
    }
    let original_resolution = resize_bilinear(
        &model_resolution,
        input_size,
        input_size,
        target_width,
        target_height,
    )?;
    let threshold_logit = probability_to_logit(mask_threshold);
    let mut mask = GrayImage::new(letterbox.original_width, letterbox.original_height);
    for y in 0..letterbox.original_height as usize {
        for x in 0..letterbox.original_width as usize {
            if original_resolution[y * target_width + x] > threshold_logit {
                mask.put_pixel(x as u32, y as u32, Luma([255]));
            }
        }
    }
    Ok(mask)
}

fn dynamic_mask_logits(candidate: &Candidate, mask_features: &ModelTensor) -> Result<Vec<f32>> {
    if candidate.kernel.len() != DYNAMIC_KERNEL_PARAMETERS {
        return Err(Error::InvalidModelOutput(format!(
            "dynamic kernel has {} parameters, expected {DYNAMIC_KERNEL_PARAMETERS}",
            candidate.kernel.len()
        )));
    }
    let height = mask_features.shape[2];
    let width = mask_features.shape[3];
    let plane = height * width;
    let expected_features = MASK_FEATURE_CHANNELS * plane;
    if mask_features.values.len() != expected_features {
        return Err(Error::InvalidModelOutput(format!(
            "mask feature tensor has {} values, expected {expected_features}",
            mask_features.values.len()
        )));
    }

    let mut logits = vec![0.0; plane];
    for y in 0..height {
        for x in 0..width {
            let position = y * width + x;
            let coord_x = x as f32 * STRIDES[0] as f32;
            let coord_y = y as f32 * STRIDES[0] as f32;
            let denominator = candidate.prior_stride * STRIDES[0] as f32;
            let mut features = [0.0_f32; MASK_FEATURE_CHANNELS + 2];
            features[0] = (candidate.prior_x - coord_x) / denominator;
            features[1] = (candidate.prior_y - coord_y) / denominator;
            for channel in 0..MASK_FEATURE_CHANNELS {
                features[channel + 2] = mask_features.values[channel * plane + position];
            }

            let mut first = [0.0_f32; DYNAMIC_CHANNELS];
            for (output, output_value) in first.iter_mut().enumerate() {
                let mut value = candidate.kernel[FIRST_LAYER_BIAS_OFFSET + output];
                for (input, feature) in features.iter().enumerate() {
                    value += candidate.kernel[output * features.len() + input] * feature;
                }
                *output_value = value.max(0.0);
            }

            let mut second = [0.0_f32; DYNAMIC_CHANNELS];
            for (output, output_value) in second.iter_mut().enumerate() {
                let mut value = candidate.kernel[SECOND_LAYER_BIAS_OFFSET + output];
                for (input, first_value) in first.iter().enumerate() {
                    let weight_index = FIRST_LAYER_WEIGHTS + output * DYNAMIC_CHANNELS + input;
                    value += candidate.kernel[weight_index] * first_value;
                }
                *output_value = value.max(0.0);
            }

            let mut value = candidate.kernel[THIRD_LAYER_BIAS_OFFSET];
            for (input, second_value) in second.iter().enumerate() {
                let weight_index = FIRST_LAYER_WEIGHTS + SECOND_LAYER_WEIGHTS + input;
                value += candidate.kernel[weight_index] * second_value;
            }
            logits[position] = value;
        }
    }
    debug_assert_eq!(
        FIRST_LAYER_WEIGHTS + SECOND_LAYER_WEIGHTS + THIRD_LAYER_WEIGHTS,
        152
    );
    Ok(logits)
}

fn sigmoid(value: f32) -> f32 {
    1.0 / (1.0 + (-value).exp())
}

fn probability_to_logit(probability: f32) -> f32 {
    if probability == 0.0 {
        f32::NEG_INFINITY
    } else if probability == 1.0 {
        f32::INFINITY
    } else {
        (probability / (1.0 - probability)).ln()
    }
}

#[derive(Clone, Copy)]
struct AxisSample {
    low: usize,
    high: usize,
    high_weight: f32,
}

fn interpolation_axis(input: usize, output: usize) -> Vec<AxisSample> {
    (0..output)
        .map(|destination| {
            let source = (destination as f32 + 0.5) * input as f32 / output as f32 - 0.5;
            let low_unclamped = source.floor() as isize;
            let high_unclamped = low_unclamped + 1;
            let high_weight = source - low_unclamped as f32;
            AxisSample {
                low: low_unclamped.clamp(0, input as isize - 1) as usize,
                high: high_unclamped.clamp(0, input as isize - 1) as usize,
                high_weight,
            }
        })
        .collect()
}

fn resize_bilinear(
    input: &[f32],
    input_width: usize,
    input_height: usize,
    output_width: usize,
    output_height: usize,
) -> Result<Vec<f32>> {
    if input_width == 0 || input_height == 0 || output_width == 0 || output_height == 0 {
        return Err(Error::InvalidConfiguration(
            "bilinear resize dimensions must be non-zero".to_owned(),
        ));
    }
    if input.len() != input_width * input_height {
        return Err(Error::InvalidModelOutput(format!(
            "bilinear resize input has {} values, expected {}",
            input.len(),
            input_width * input_height
        )));
    }
    let x_samples = interpolation_axis(input_width, output_width);
    let y_samples = interpolation_axis(input_height, output_height);
    let mut output = vec![0.0; output_width * output_height];
    for (y, y_sample) in y_samples.iter().enumerate() {
        let wy = y_sample.high_weight;
        for (x, x_sample) in x_samples.iter().enumerate() {
            let wx = x_sample.high_weight;
            let top_left = input[y_sample.low * input_width + x_sample.low];
            let top_right = input[y_sample.low * input_width + x_sample.high];
            let bottom_left = input[y_sample.high * input_width + x_sample.low];
            let bottom_right = input[y_sample.high * input_width + x_sample.high];
            let top = top_left + (top_right - top_left) * wx;
            let bottom = bottom_left + (bottom_right - bottom_left) * wx;
            output[y * output_width + x] = top + (bottom - top) * wy;
        }
    }
    Ok(output)
}

fn resize_rgb_bilinear(input: &RgbImage, output_width: u32, output_height: u32) -> RgbImage {
    let x_samples = interpolation_axis(input.width() as usize, output_width as usize);
    let y_samples = interpolation_axis(input.height() as usize, output_height as usize);
    let mut output = RgbImage::new(output_width, output_height);
    for (y, y_sample) in y_samples.iter().enumerate() {
        let wy = y_sample.high_weight;
        for (x, x_sample) in x_samples.iter().enumerate() {
            let wx = x_sample.high_weight;
            let top_left = input.get_pixel(x_sample.low as u32, y_sample.low as u32);
            let top_right = input.get_pixel(x_sample.high as u32, y_sample.low as u32);
            let bottom_left = input.get_pixel(x_sample.low as u32, y_sample.high as u32);
            let bottom_right = input.get_pixel(x_sample.high as u32, y_sample.high as u32);
            let mut pixel = [0_u8; 3];
            for channel in 0..3 {
                let top = top_left[channel] as f32
                    + (top_right[channel] as f32 - top_left[channel] as f32) * wx;
                let bottom = bottom_left[channel] as f32
                    + (bottom_right[channel] as f32 - bottom_left[channel] as f32) * wx;
                pixel[channel] = (top + (bottom - top) * wy).round().clamp(0.0, 255.0) as u8;
            }
            output.put_pixel(x as u32, y as u32, Rgb(pixel));
        }
    }
    output
}

fn bgr_to_chw_0_255(image: &RgbImage) -> Vec<f32> {
    let plane = image.width() as usize * image.height() as usize;
    let mut tensor = vec![0.0; plane * 3];
    for (index, pixel) in image.pixels().enumerate() {
        tensor[index] = f32::from(pixel[2]);
        tensor[plane + index] = f32::from(pixel[1]);
        tensor[plane * 2 + index] = f32::from(pixel[0]);
    }
    tensor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_letterbox() -> Letterbox {
        Letterbox::new(
            &DynamicImage::ImageRgb8(ImageBuffer::from_pixel(100, 50, Rgb([1, 2, 3]))),
            100,
        )
        .unwrap()
    }

    #[test]
    fn letterbox_pads_bottom_and_right_and_preserves_bgr_channels() {
        let letterbox = test_letterbox();
        assert_eq!((letterbox.scale_x, letterbox.scale_y), (1.0, 1.0));
        assert_eq!(letterbox.image.get_pixel(0, 0), &Rgb([1, 2, 3]));
        assert_eq!(letterbox.image.get_pixel(0, 50), &Rgb([114, 114, 114]));
        let tensor = bgr_to_chw_0_255(&letterbox.image);
        let plane = 100 * 100;
        assert_eq!(tensor[0], 3.0);
        assert_eq!(tensor[plane], 2.0);
        assert_eq!(tensor[plane * 2], 1.0);
    }

    #[test]
    fn decodes_distance_box_from_checkpoint_zero_offset_prior() {
        let level = LevelOutputs {
            cls: ModelTensor {
                shape: vec![1, 1, 1, 1],
                values: vec![4.0],
            },
            bbox: ModelTensor {
                shape: vec![1, 4, 1, 1],
                values: vec![1.0, 2.0, 3.0, 4.0],
            },
            kernel: ModelTensor {
                shape: vec![1, DYNAMIC_KERNEL_PARAMETERS, 1, 1],
                values: vec![0.0; DYNAMIC_KERNEL_PARAMETERS],
            },
            stride: 8,
            height: 1,
            width: 1,
        };
        let candidates = decode_level(&level, &test_letterbox(), 0.5).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].bounding_box,
            BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 3.0,
                height: 4.0,
            }
        );
        assert!((candidates[0].confidence - sigmoid(4.0)).abs() < 1e-6);
    }

    #[test]
    fn nms_keeps_the_higher_scoring_overlapping_candidate() {
        let candidate = |confidence, x| Candidate {
            confidence,
            bounding_box: BoundingBox {
                x,
                y: 10.0,
                width: 50.0,
                height: 50.0,
            },
            prior_x: 0.0,
            prior_y: 0.0,
            prior_stride: 8.0,
            kernel: vec![0.0; DYNAMIC_KERNEL_PARAMETERS],
        };
        let selected =
            non_maximum_suppression(vec![candidate(0.8, 12.0), candidate(0.9, 10.0)], 0.6);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].confidence, 0.9);
    }

    #[test]
    fn dynamic_mask_uses_final_bias_for_every_pixel() {
        let mut kernel = vec![0.0; DYNAMIC_KERNEL_PARAMETERS];
        kernel[THIRD_LAYER_BIAS_OFFSET] = 1.25;
        let candidate = Candidate {
            confidence: 0.9,
            bounding_box: BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
            prior_x: 4.0,
            prior_y: 4.0,
            prior_stride: 8.0,
            kernel,
        };
        let features = ModelTensor {
            shape: vec![1, MASK_FEATURE_CHANNELS, 2, 2],
            values: vec![0.0; MASK_FEATURE_CHANNELS * 4],
        };
        assert_eq!(
            dynamic_mask_logits(&candidate, &features).unwrap(),
            vec![1.25; 4]
        );
    }

    #[test]
    fn dynamic_mask_coordinates_use_checkpoint_zero_offset_grid() {
        let mut kernel = vec![0.0; DYNAMIC_KERNEL_PARAMETERS];
        kernel[0] = 1.0;
        kernel[FIRST_LAYER_WEIGHTS] = 1.0;
        kernel[FIRST_LAYER_WEIGHTS + SECOND_LAYER_WEIGHTS] = 1.0;
        let candidate = Candidate {
            confidence: 0.9,
            bounding_box: BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
            prior_x: 8.0,
            prior_y: 0.0,
            prior_stride: 8.0,
            kernel,
        };
        let features = ModelTensor {
            shape: vec![1, MASK_FEATURE_CHANNELS, 1, 2],
            values: vec![0.0; MASK_FEATURE_CHANNELS * 2],
        };

        assert_eq!(
            dynamic_mask_logits(&candidate, &features).unwrap(),
            vec![0.125, 0.0]
        );
    }

    #[test]
    fn bilinear_resize_matches_half_pixel_interpolation() {
        let resized = resize_bilinear(&[0.0, 2.0, 4.0, 6.0], 2, 2, 4, 4).unwrap();
        assert_eq!(resized[0], 0.0);
        assert_eq!(resized[3], 2.0);
        assert_eq!(resized[12], 4.0);
        assert_eq!(resized[15], 6.0);
        assert!((resized[5] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn mask_restoration_handles_independently_rounded_letterbox_axes() {
        let letterbox = Letterbox::new(
            &DynamicImage::ImageRgb8(ImageBuffer::from_pixel(103, 73, Rgb([1, 2, 3]))),
            32,
        )
        .unwrap();
        let mut kernel = vec![0.0; DYNAMIC_KERNEL_PARAMETERS];
        kernel[THIRD_LAYER_BIAS_OFFSET] = 1.0;
        let candidate = Candidate {
            confidence: 0.9,
            bounding_box: BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 103.0,
                height: 73.0,
            },
            prior_x: 0.0,
            prior_y: 0.0,
            prior_stride: 8.0,
            kernel,
        };
        let features = ModelTensor {
            shape: vec![1, MASK_FEATURE_CHANNELS, 1, 1],
            values: vec![0.0; MASK_FEATURE_CHANNELS],
        };

        let mask = render_mask(&candidate, &features, &letterbox, 0.5).unwrap();

        assert_eq!(mask.dimensions(), (103, 73));
        assert_eq!(mask.get_pixel(0, 0), &Luma([255]));
        assert_eq!(mask.get_pixel(102, 72), &Luma([255]));
    }

    #[test]
    fn options_reject_zero_instance_limit() {
        let options = SegmentationOptions {
            max_instances: 0,
            ..SegmentationOptions::default()
        };
        assert!(matches!(
            options.validate(),
            Err(Error::InvalidConfiguration(message)) if message.contains("max_instances")
        ));
    }

    #[test]
    fn anime_options_default_to_sixty_percent_confidence() {
        assert_eq!(AnimeInsSegOptions::default().confidence_threshold, 0.6);
    }

    #[test]
    fn cross_model_nms_keeps_higher_scoring_duplicate_and_non_overlapping_instance() {
        let anime = vec![
            Segmentation {
                confidence: 0.8,
                bounding_box: BoundingBox {
                    x: 10.0,
                    y: 10.0,
                    width: 50.0,
                    height: 50.0,
                },
                mask: GrayImage::from_pixel(2, 2, Luma([255])),
            },
            Segmentation {
                confidence: 0.7,
                bounding_box: BoundingBox {
                    x: 100.0,
                    y: 100.0,
                    width: 20.0,
                    height: 20.0,
                },
                mask: GrayImage::from_pixel(2, 2, Luma([255])),
            },
        ];
        let yolo = vec![Yolo11Segmentation {
            class_id: 0,
            class_name: "person".to_owned(),
            confidence: 0.9,
            bounding_box: Yolo11BoundingBox {
                x: 12.0,
                y: 12.0,
                width: 50.0,
                height: 50.0,
            },
            mask: GrayImage::from_pixel(2, 2, Luma([128])),
        }];

        let merged = merge_segmentations(&anime, &yolo, 0.6).unwrap();

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].backend, SegmentationBackend::Yolo11);
        assert_eq!(merged[0].confidence, 0.9);
        assert_eq!(merged[0].mask.get_pixel(0, 0), &Luma([128]));
        assert_eq!(merged[1].backend, SegmentationBackend::AnimeInsSeg);
        assert_eq!(merged[1].bounding_box.x, 100.0);
    }

    #[test]
    fn cross_model_nms_suppresses_duplicate_masks_when_boxes_differ() {
        let shared_mask = GrayImage::from_pixel(4, 4, Luma([255]));
        let anime = vec![Segmentation {
            confidence: 0.95,
            bounding_box: BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            mask: shared_mask.clone(),
        }];
        let yolo = vec![Yolo11Segmentation {
            class_id: 0,
            class_name: "person".to_owned(),
            confidence: 0.9,
            bounding_box: Yolo11BoundingBox {
                x: 25.0,
                y: 25.0,
                width: 50.0,
                height: 50.0,
            },
            mask: shared_mask,
        }];

        let merged = merge_segmentations(&anime, &yolo, 0.6).unwrap();

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].backend, SegmentationBackend::AnimeInsSeg);
        assert_eq!(merged[0].confidence, 0.95);
    }

    #[test]
    fn cross_model_nms_rejects_non_finite_or_out_of_range_thresholds() {
        for threshold in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
            assert!(matches!(
                merge_segmentations(&[], &[], threshold),
                Err(Error::InvalidConfiguration(message))
                    if message.contains("iou_threshold")
            ));
        }
    }
}
