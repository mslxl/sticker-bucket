#[cfg(feature = "onnx")]
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use image::DynamicImage;
#[cfg(any(feature = "onnx", test))]
use image::{Rgb, RgbImage, imageops::FilterType};

#[cfg(any(feature = "onnx", test))]
use crate::Error;
use crate::Result;
#[cfg(feature = "onnx")]
use crate::{ExecutionPolicy, FeatureSchema, Provider};

pub trait Tagger: Send {
    fn feature_schema_id(&self) -> &str;
    fn tag(&mut self, image: &DynamicImage) -> Result<Vec<f32>>;
}

#[cfg(feature = "onnx")]
pub struct MlDanbooruTagger {
    session: ort::session::Session,
    schema: FeatureSchema,
    selected_class_indices: Vec<usize>,
}

#[cfg(not(feature = "onnx"))]
pub struct MlDanbooruTagger {
    _private: (),
}

#[cfg(feature = "onnx")]
impl std::fmt::Debug for MlDanbooruTagger {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MlDanbooruTagger")
            .field("feature_schema", &self.schema.id)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "onnx")]
impl MlDanbooruTagger {
    pub fn load(
        model_path: impl AsRef<Path>,
        classes_path: impl AsRef<Path>,
        schema: FeatureSchema,
        execution_policy: ExecutionPolicy,
    ) -> Result<Self> {
        let classes: Vec<String> =
            serde_json::from_reader(BufReader::new(File::open(classes_path)?))?;
        Self::load_with_classes(model_path, &classes, schema, execution_policy)
    }

    pub fn load_with_classes(
        model_path: impl AsRef<Path>,
        classes: &[String],
        schema: FeatureSchema,
        execution_policy: ExecutionPolicy,
    ) -> Result<Self> {
        schema.validate()?;
        let class_indices: HashMap<&str, usize> = classes
            .iter()
            .enumerate()
            .map(|(index, class)| (class.as_str(), index))
            .collect();
        let selected_class_indices = schema
            .features
            .iter()
            .map(|feature| {
                class_indices
                    .get(feature.tag.as_str())
                    .copied()
                    .ok_or_else(|| {
                        Error::InvalidFeatureSchema(format!(
                            "model classes do not contain optimized feature `{}`",
                            feature.tag
                        ))
                    })
            })
            .collect::<Result<Vec<_>>>()?;

        let providers = execution_providers(&execution_policy);
        let builder = ort::session::Session::builder()?;
        let mut builder = if providers.is_empty() {
            builder
        } else {
            builder
                .with_execution_providers(providers)
                .map_err(|error| {
                    Error::InvalidModelOutput(format!(
                        "failed to configure ONNX execution providers: {error}"
                    ))
                })?
        };
        let session = builder.commit_from_file(model_path)?;
        Ok(Self {
            session,
            schema,
            selected_class_indices,
        })
    }
}

#[cfg(feature = "onnx")]
impl Tagger for MlDanbooruTagger {
    fn feature_schema_id(&self) -> &str {
        &self.schema.id
    }

    fn tag(&mut self, image: &DynamicImage) -> Result<Vec<f32>> {
        let prepared = prepare_image(image, 512)?;
        let height = prepared.height() as usize;
        let width = prepared.width() as usize;
        let tensor_data = rgb_to_chw(&prepared);
        let tensor = ort::value::Tensor::from_array(([1_usize, 3, height, width], tensor_data))?;
        let outputs = self.session.run(ort::inputs![tensor])?;
        if outputs.len() == 0 {
            return Err(Error::InvalidModelOutput(
                "model returned no outputs".to_owned(),
            ));
        }
        let (_, logits) = outputs[0].try_extract_tensor::<f32>()?;
        let largest_index = self
            .selected_class_indices
            .iter()
            .copied()
            .max()
            .ok_or_else(|| Error::InvalidModelOutput("feature index list is empty".to_owned()))?;
        if largest_index >= logits.len() {
            return Err(Error::InvalidModelOutput(format!(
                "model returned {} classes but feature schema needs class index {largest_index}",
                logits.len()
            )));
        }

        self.selected_class_indices
            .iter()
            .map(|index| {
                let probability = sigmoid(logits[*index]);
                if !probability.is_finite() {
                    return Err(Error::InvalidModelOutput(format!(
                        "class {index} produced non-finite probability"
                    )));
                }
                Ok(if probability >= self.schema.threshold {
                    probability
                } else {
                    0.0
                })
            })
            .collect()
    }
}

#[cfg(feature = "onnx")]
fn execution_providers(policy: &ExecutionPolicy) -> Vec<ort::ep::ExecutionProviderDispatch> {
    use ort::ep::{CUDA, CoreML, DirectML};

    let providers = match policy {
        ExecutionPolicy::Auto => {
            #[cfg(target_vendor = "apple")]
            let providers = vec![Provider::CoreMl];
            #[cfg(target_os = "windows")]
            let providers = vec![Provider::DirectMl];
            #[cfg(all(not(target_vendor = "apple"), not(target_os = "windows")))]
            let providers = vec![Provider::Cuda];
            providers
        }
        ExecutionPolicy::Cpu => Vec::new(),
        ExecutionPolicy::Prefer(providers) => providers.clone(),
        ExecutionPolicy::Require(provider) => vec![*provider],
    };
    let required = matches!(policy, ExecutionPolicy::Require(_));
    providers
        .into_iter()
        .filter_map(|provider| {
            let dispatch = match provider {
                Provider::Cpu => return None,
                Provider::CoreMl => CoreML::default().build(),
                Provider::DirectMl => DirectML::default().build(),
                Provider::Cuda => CUDA::default().build(),
            };
            Some(if required {
                dispatch.error_on_failure()
            } else {
                dispatch.fail_silently()
            })
        })
        .collect()
}

#[cfg(any(feature = "onnx", test))]
pub(crate) fn prepare_image(image: &DynamicImage, size: u32) -> Result<RgbImage> {
    if image.width() == 0 || image.height() == 0 {
        return Err(Error::InvalidModelOutput(
            "image dimensions must be non-zero".to_owned(),
        ));
    }
    let rgba = image.to_rgba8();
    let mut rgb = RgbImage::new(rgba.width(), rgba.height());
    for (source, target) in rgba.pixels().zip(rgb.pixels_mut()) {
        let alpha = u16::from(source[3]);
        let blend = |channel: u8| -> u8 {
            let value = u16::from(channel) * alpha + 255 * (255 - alpha);
            ((value + 127) / 255) as u8
        };
        *target = Rgb([blend(source[0]), blend(source[1]), blend(source[2])]);
    }

    let minimum = image.width().min(image.height());
    let target_width =
        ((u64::from(image.width()) * u64::from(size) / u64::from(minimum)) as u32 / 4) * 4;
    let target_height =
        ((u64::from(image.height()) * u64::from(size) / u64::from(minimum)) as u32 / 4) * 4;
    if target_width == 0 || target_height == 0 {
        return Err(Error::InvalidModelOutput(
            "resized image dimensions became zero".to_owned(),
        ));
    }
    Ok(image::imageops::resize(
        &rgb,
        target_width,
        target_height,
        FilterType::Triangle,
    ))
}

#[cfg(feature = "onnx")]
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

#[cfg(feature = "onnx")]
fn sigmoid(value: f32) -> f32 {
    if value >= 0.0 {
        1.0 / (1.0 + (-value).exp())
    } else {
        let exponential = value.exp();
        exponential / (1.0 + exponential)
    }
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, Rgba, RgbaImage};

    #[test]
    fn fills_transparency_with_white_and_preserves_upstream_resize_shape() {
        let mut image = RgbaImage::new(8, 4);
        image.put_pixel(0, 0, Rgba([10, 20, 30, 0]));
        image.put_pixel(1, 0, Rgba([10, 20, 30, 255]));

        let prepared = super::prepare_image(&DynamicImage::ImageRgba8(image), 4).unwrap();

        assert_eq!(prepared.dimensions(), (8, 4));
        assert_eq!(prepared.get_pixel(0, 0).0, [255, 255, 255]);
        assert_eq!(prepared.get_pixel(1, 0).0, [10, 20, 30]);
    }
}
