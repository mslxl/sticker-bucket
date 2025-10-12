//! Local ONNX inference for YOLO11 segmentation exports.
//!
//! The model is expected to have the standard Ultralytics segmentation outputs:
//! a `[1, 4 + classes + mask_coefficients, candidates]` prediction tensor and a
//! `[1, mask_coefficients, mask_height, mask_width]` prototype tensor.

use std::path::Path;

use image::{DynamicImage, GrayImage, ImageBuffer, Luma, Rgb, RgbImage, imageops::FilterType};
use ort::{session::Session, value::Tensor};
use serde::Serialize;
use thiserror::Error;

const DEFAULT_INPUT_SIZE: u32 = 640;
const MASK_THRESHOLD: u8 = 128;

/// Errors returned by [`Yolo11Seg`].
#[derive(Debug, Error)]
pub enum Error {
    #[error("ONNX Runtime operation failed: {0}")]
    Onnx(#[from] ort::Error),

    #[error("image operation failed: {0}")]
    Image(#[from] image::ImageError),

    #[error("invalid segmentation configuration: {0}")]
    InvalidConfiguration(String),

    #[error("model output is incompatible with YOLO segmentation: {0}")]
    InvalidModelOutput(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// A rectangle in original-image pixel coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

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

/// A single detected object and its full-resolution binary mask.
#[derive(Debug)]
pub struct Segmentation {
    pub class_id: usize,
    pub class_name: String,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
    pub mask: GrayImage,
}

/// Inference and post-processing parameters.
#[derive(Clone, Debug)]
pub struct SegmentationOptions {
    /// Square resolution used when the ONNX model was exported.
    pub input_size: u32,
    pub confidence_threshold: f32,
    pub iou_threshold: f32,
    /// Class names in exact model-output order. Defaults to COCO's 80 labels.
    pub labels: Vec<String>,
}

impl Default for SegmentationOptions {
    fn default() -> Self {
        Self {
            input_size: DEFAULT_INPUT_SIZE,
            confidence_threshold: 0.25,
            iou_threshold: 0.7,
            labels: coco_labels().into_iter().map(str::to_owned).collect(),
        }
    }
}

impl SegmentationOptions {
    fn validate(&self) -> Result<()> {
        if self.input_size == 0 {
            return Err(Error::InvalidConfiguration(
                "input_size must be positive".to_owned(),
            ));
        }
        for (name, value) in [
            ("confidence_threshold", self.confidence_threshold),
            ("iou_threshold", self.iou_threshold),
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(Error::InvalidConfiguration(format!(
                    "{name} must be finite and in [0, 1], got {value}"
                )));
            }
        }
        if self.labels.iter().any(|label| label.trim().is_empty()) {
            return Err(Error::InvalidConfiguration(
                "labels must not contain empty values".to_owned(),
            ));
        }
        Ok(())
    }
}

/// A loaded YOLO11 segmentation model.
pub struct Yolo11Seg {
    session: Session,
    options: SegmentationOptions,
}

impl Yolo11Seg {
    /// Loads an ONNX model exported from a YOLO11 segmentation checkpoint.
    pub fn load(model_path: impl AsRef<Path>, options: SegmentationOptions) -> Result<Self> {
        options.validate()?;
        let session = Session::builder()?.commit_from_file(model_path)?;
        Ok(Self { session, options })
    }

    /// Segments every detected object in `image`.
    pub fn segment(&mut self, image: &DynamicImage) -> Result<Vec<Segmentation>> {
        let letterbox = Letterbox::new(image, self.options.input_size)?;
        let size = self.options.input_size as usize;
        let tensor = Tensor::from_array(([1_usize, 3, size, size], rgb_to_chw(&letterbox.image)))?;
        let outputs = self.session.run(ort::inputs![tensor])?;
        let ModelOutputs {
            prediction,
            prototypes,
        } = ModelOutputs::from_session_outputs(&outputs)?;
        let candidates = decode_candidates(
            prediction,
            prototypes.shape[1],
            self.options.confidence_threshold,
            &self.options.labels,
            &letterbox,
        )?;
        let selected = non_maximum_suppression(candidates, self.options.iou_threshold);

        selected
            .into_iter()
            .map(|candidate| candidate.into_segmentation(&prototypes, &letterbox))
            .collect()
    }
}

#[derive(Debug)]
struct Letterbox {
    image: RgbImage,
    original_width: u32,
    original_height: u32,
    scale: f32,
    pad_x: u32,
    pad_y: u32,
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
        let pad_x = (input_size - resized_width) / 2;
        let pad_y = (input_size - resized_height) / 2;
        let resized = image::imageops::resize(
            &image.to_rgb8(),
            resized_width,
            resized_height,
            FilterType::Triangle,
        );
        let mut output = ImageBuffer::from_pixel(input_size, input_size, Rgb([114, 114, 114]));
        image::imageops::overlay(&mut output, &resized, i64::from(pad_x), i64::from(pad_y));
        Ok(Self {
            image: output,
            original_width,
            original_height,
            scale,
            pad_x,
            pad_y,
        })
    }

    fn to_original_box(
        &self,
        center_x: f32,
        center_y: f32,
        width: f32,
        height: f32,
    ) -> BoundingBox {
        let left = ((center_x - width / 2.0) - self.pad_x as f32) / self.scale;
        let top = ((center_y - height / 2.0) - self.pad_y as f32) / self.scale;
        let right = ((center_x + width / 2.0) - self.pad_x as f32) / self.scale;
        let bottom = ((center_y + height / 2.0) - self.pad_y as f32) / self.scale;
        let left = left.clamp(0.0, self.original_width as f32);
        let top = top.clamp(0.0, self.original_height as f32);
        let right = right.clamp(left, self.original_width as f32);
        let bottom = bottom.clamp(top, self.original_height as f32);
        BoundingBox {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }
}

struct ModelTensor {
    shape: Vec<usize>,
    values: Vec<f32>,
}

struct ModelOutputs {
    prediction: ModelTensor,
    prototypes: ModelTensor,
}

impl ModelOutputs {
    fn from_session_outputs(outputs: &ort::session::SessionOutputs<'_>) -> Result<Self> {
        let mut prediction = None;
        let mut prototypes = None;
        for index in 0..outputs.len() {
            let (shape, values) = outputs[index].try_extract_tensor::<f32>()?;
            let tensor = ModelTensor {
                shape: shape.iter().map(|dimension| *dimension as usize).collect(),
                values: values.to_vec(),
            };
            match tensor.shape.len() {
                3 => prediction = Some(tensor),
                4 => prototypes = Some(tensor),
                dimensions => {
                    return Err(Error::InvalidModelOutput(format!(
                        "output {index} has {dimensions} dimensions; expected 3 or 4"
                    )));
                }
            }
        }
        let prediction = prediction.ok_or_else(|| {
            Error::InvalidModelOutput("missing three-dimensional prediction tensor".to_owned())
        })?;
        let prototypes = prototypes.ok_or_else(|| {
            Error::InvalidModelOutput("missing four-dimensional mask prototype tensor".to_owned())
        })?;
        if prototypes.shape[0] != 1
            || prototypes.shape[1] == 0
            || prototypes.shape[2] == 0
            || prototypes.shape[3] == 0
        {
            return Err(Error::InvalidModelOutput(format!(
                "invalid prototype tensor shape {:?}",
                prototypes.shape
            )));
        }
        Ok(Self {
            prediction,
            prototypes,
        })
    }
}

#[derive(Debug)]
struct Candidate {
    class_id: usize,
    class_name: String,
    confidence: f32,
    bounding_box: BoundingBox,
    coefficients: Vec<f32>,
}

impl Candidate {
    fn into_segmentation(
        self,
        prototypes: &ModelTensor,
        letterbox: &Letterbox,
    ) -> Result<Segmentation> {
        let mask = render_mask(&self.coefficients, prototypes, letterbox, self.bounding_box)?;
        Ok(Segmentation {
            class_id: self.class_id,
            class_name: self.class_name,
            confidence: self.confidence,
            bounding_box: self.bounding_box,
            mask,
        })
    }
}

fn decode_candidates(
    prediction: ModelTensor,
    mask_channels: usize,
    confidence_threshold: f32,
    labels: &[String],
    letterbox: &Letterbox,
) -> Result<Vec<Candidate>> {
    if prediction.shape[0] != 1 {
        return Err(Error::InvalidModelOutput(format!(
            "prediction batch size must be 1, got {}",
            prediction.shape[0]
        )));
    }
    let minimum_features = 4 + mask_channels;
    let expected_features = minimum_features + labels.len();
    let (features, candidates, feature_first) = if prediction.shape[1] == expected_features {
        (prediction.shape[1], prediction.shape[2], true)
    } else if prediction.shape[2] == expected_features {
        (prediction.shape[2], prediction.shape[1], false)
    } else if prediction.shape[1] >= minimum_features && prediction.shape[2] < minimum_features {
        (prediction.shape[1], prediction.shape[2], true)
    } else if prediction.shape[2] >= minimum_features && prediction.shape[1] < minimum_features {
        (prediction.shape[2], prediction.shape[1], false)
    } else if prediction.shape[1] >= minimum_features && prediction.shape[2] >= minimum_features {
        // Standard YOLO exports have many more candidates than output features.
        // Picking the smaller dimension here keeps custom-class failures explicit.
        if prediction.shape[1] <= prediction.shape[2] {
            (prediction.shape[1], prediction.shape[2], true)
        } else {
            (prediction.shape[2], prediction.shape[1], false)
        }
    } else {
        return Err(Error::InvalidModelOutput(format!(
            "cannot identify feature dimension in prediction shape {:?}",
            prediction.shape
        )));
    };
    let class_count = features - 4 - mask_channels;
    if class_count == 0 {
        return Err(Error::InvalidModelOutput(
            "prediction tensor has no class scores".to_owned(),
        ));
    }
    if class_count != labels.len() {
        return Err(Error::InvalidModelOutput(format!(
            "model provides {class_count} classes but {} labels were configured",
            labels.len()
        )));
    }
    if prediction.values.len() != features * candidates {
        return Err(Error::InvalidModelOutput(format!(
            "prediction tensor shape {:?} does not match {} values",
            prediction.shape,
            prediction.values.len()
        )));
    }

    let value_at = |feature: usize, candidate: usize| {
        if feature_first {
            prediction.values[feature * candidates + candidate]
        } else {
            prediction.values[candidate * features + feature]
        }
    };
    let mut decoded = Vec::new();
    for candidate_index in 0..candidates {
        let (class_id, confidence) = (0..class_count)
            .map(|class_id| (class_id, value_at(4 + class_id, candidate_index)))
            .max_by(|(_, left), (_, right)| left.total_cmp(right))
            .ok_or_else(|| Error::InvalidModelOutput("model has no class scores".to_owned()))?;
        if !confidence.is_finite() || confidence < confidence_threshold {
            continue;
        }
        let center_x = value_at(0, candidate_index);
        let center_y = value_at(1, candidate_index);
        let width = value_at(2, candidate_index);
        let height = value_at(3, candidate_index);
        if ![center_x, center_y, width, height]
            .into_iter()
            .all(f32::is_finite)
            || width <= 0.0
            || height <= 0.0
        {
            continue;
        }
        let bounding_box = letterbox.to_original_box(center_x, center_y, width, height);
        if bounding_box.width <= 0.0 || bounding_box.height <= 0.0 {
            continue;
        }
        let coefficients = (0..mask_channels)
            .map(|channel| value_at(4 + class_count + channel, candidate_index))
            .collect::<Vec<_>>();
        if coefficients.iter().any(|value| !value.is_finite()) {
            continue;
        }
        decoded.push(Candidate {
            class_id,
            class_name: labels[class_id].clone(),
            confidence,
            bounding_box,
            coefficients,
        });
    }
    Ok(decoded)
}

fn non_maximum_suppression(mut candidates: Vec<Candidate>, iou_threshold: f32) -> Vec<Candidate> {
    candidates.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));
    let mut selected: Vec<Candidate> = Vec::new();
    'candidate: for candidate in candidates {
        for existing in &selected {
            if candidate.class_id == existing.class_id
                && candidate
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
    coefficients: &[f32],
    prototypes: &ModelTensor,
    letterbox: &Letterbox,
    bounding_box: BoundingBox,
) -> Result<GrayImage> {
    let channels = prototypes.shape[1];
    let prototype_height = prototypes.shape[2];
    let prototype_width = prototypes.shape[3];
    if coefficients.len() != channels
        || prototypes.values.len() != channels * prototype_height * prototype_width
    {
        return Err(Error::InvalidModelOutput(
            "mask coefficients and prototypes are inconsistent".to_owned(),
        ));
    }
    let mut low_resolution = GrayImage::new(prototype_width as u32, prototype_height as u32);
    for y in 0..prototype_height {
        for x in 0..prototype_width {
            let position = y * prototype_width + x;
            let logit = (0..channels)
                .map(|channel| {
                    coefficients[channel]
                        * prototypes.values[channel * prototype_height * prototype_width + position]
                })
                .sum::<f32>();
            let probability = 1.0 / (1.0 + (-logit).exp());
            low_resolution.put_pixel(
                x as u32,
                y as u32,
                Luma([(probability * 255.0).round() as u8]),
            );
        }
    }
    let input_size = letterbox.image.width();
    let model_mask = image::imageops::resize(
        &low_resolution,
        input_size,
        input_size,
        FilterType::Triangle,
    );
    let mut output = GrayImage::new(letterbox.original_width, letterbox.original_height);
    let min_x = bounding_box.x.floor().max(0.0) as u32;
    let min_y = bounding_box.y.floor().max(0.0) as u32;
    let max_x = (bounding_box.x + bounding_box.width)
        .ceil()
        .min(letterbox.original_width as f32) as u32;
    let max_y = (bounding_box.y + bounding_box.height)
        .ceil()
        .min(letterbox.original_height as f32) as u32;
    for y in min_y..max_y {
        let model_y = (y as f32 * letterbox.scale + letterbox.pad_y as f32)
            .round()
            .clamp(0.0, (input_size - 1) as f32) as u32;
        for x in min_x..max_x {
            let model_x = (x as f32 * letterbox.scale + letterbox.pad_x as f32)
                .round()
                .clamp(0.0, (input_size - 1) as f32) as u32;
            let value = model_mask.get_pixel(model_x, model_y)[0];
            if value >= MASK_THRESHOLD {
                output.put_pixel(x, y, Luma([255]));
            }
        }
    }
    Ok(output)
}

fn rgb_to_chw(image: &RgbImage) -> Vec<f32> {
    let plane = image.width() as usize * image.height() as usize;
    let mut tensor = vec![0.0; plane * 3];
    for (index, pixel) in image.pixels().enumerate() {
        tensor[index] = f32::from(pixel[0]) / 255.0;
        tensor[plane + index] = f32::from(pixel[1]) / 255.0;
        tensor[plane * 2 + index] = f32::from(pixel[2]) / 255.0;
    }
    tensor
}

fn coco_labels() -> [&'static str; 80] {
    [
        "person",
        "bicycle",
        "car",
        "motorcycle",
        "airplane",
        "bus",
        "train",
        "truck",
        "boat",
        "traffic light",
        "fire hydrant",
        "stop sign",
        "parking meter",
        "bench",
        "bird",
        "cat",
        "dog",
        "horse",
        "sheep",
        "cow",
        "elephant",
        "bear",
        "zebra",
        "giraffe",
        "backpack",
        "umbrella",
        "handbag",
        "tie",
        "suitcase",
        "frisbee",
        "skis",
        "snowboard",
        "sports ball",
        "kite",
        "baseball bat",
        "baseball glove",
        "skateboard",
        "surfboard",
        "tennis racket",
        "bottle",
        "wine glass",
        "cup",
        "fork",
        "knife",
        "spoon",
        "bowl",
        "banana",
        "apple",
        "sandwich",
        "orange",
        "broccoli",
        "carrot",
        "hot dog",
        "pizza",
        "donut",
        "cake",
        "chair",
        "couch",
        "potted plant",
        "bed",
        "dining table",
        "toilet",
        "tv",
        "laptop",
        "mouse",
        "remote",
        "keyboard",
        "cell phone",
        "microwave",
        "oven",
        "toaster",
        "sink",
        "refrigerator",
        "book",
        "clock",
        "vase",
        "scissors",
        "teddy bear",
        "hair drier",
        "toothbrush",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn letterbox() -> Letterbox {
        Letterbox::new(&DynamicImage::ImageRgb8(RgbImage::new(100, 100)), 100).unwrap()
    }

    #[test]
    fn decodes_feature_first_predictions_and_removes_overlapping_duplicate() {
        let prediction = ModelTensor {
            shape: vec![1, 7, 2],
            values: vec![
                50.0, 51.0, // center x
                50.0, 51.0, // center y
                40.0, 40.0, // width
                40.0, 40.0, // height
                0.90, 0.80, // one class score
                1.0, 0.5, // mask coefficient 0
                0.0, 0.5, // mask coefficient 1
            ],
        };
        let labels = vec!["person".to_owned()];
        let candidates = decode_candidates(prediction, 2, 0.25, &labels, &letterbox()).unwrap();
        let selected = non_maximum_suppression(candidates, 0.5);

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].class_name, "person");
        assert_eq!(selected[0].confidence, 0.90);
        assert_eq!(
            selected[0].bounding_box,
            BoundingBox {
                x: 30.0,
                y: 30.0,
                width: 40.0,
                height: 40.0
            }
        );
        assert_eq!(selected[0].coefficients, vec![1.0, 0.0]);
    }

    #[test]
    fn validates_label_count_against_model_classes() {
        let prediction = ModelTensor {
            shape: vec![1, 7, 1],
            values: vec![50.0, 50.0, 40.0, 40.0, 0.9, 1.0, 0.0],
        };
        let error = decode_candidates(prediction, 2, 0.25, &[], &letterbox()).unwrap_err();

        assert!(
            matches!(error, Error::InvalidModelOutput(message) if message.contains("1 classes but 0 labels"))
        );
    }
}
