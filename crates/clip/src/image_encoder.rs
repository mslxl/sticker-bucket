use std::path::Path;

use image::{DynamicImage, RgbImage, imageops::FilterType};
use ort::{session::Session, value::Tensor};

use crate::{
    Embedding, Error, ExecutionPolicy, ImageResize, ModelManifest, Result,
    execution::session_builder, text_encoder::embeddings_from_outputs,
};

pub struct ImageEncoder {
    session: Session,
    model_id: String,
    dimension: usize,
    image_size: u32,
    resize: ImageResize,
    mean: [f32; 3],
    std: [f32; 3],
}

impl std::fmt::Debug for ImageEncoder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ImageEncoder")
            .field("model_id", &self.model_id)
            .field("dimension", &self.dimension)
            .field("image_size", &self.image_size)
            .finish_non_exhaustive()
    }
}

impl ImageEncoder {
    pub fn load(directory: impl AsRef<Path>, policy: ExecutionPolicy) -> Result<Self> {
        let directory = directory.as_ref();
        let manifest = ModelManifest::from_directory(directory)?;
        Self::load_with_manifest(directory, &manifest, policy)
    }

    pub(crate) fn load_with_manifest(
        directory: &Path,
        manifest: &ModelManifest,
        policy: ExecutionPolicy,
    ) -> Result<Self> {
        let session =
            session_builder(&policy)?.commit_from_file(directory.join(&manifest.image_model))?;
        Ok(Self {
            session,
            model_id: manifest.id.clone(),
            dimension: manifest.embedding_dimension,
            image_size: manifest.image_size,
            resize: manifest.image_resize,
            mean: manifest.image_mean,
            std: manifest.image_std,
        })
    }

    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn encode(&mut self, image: &DynamicImage) -> Result<Embedding> {
        let mut embeddings = self.encode_batch(&[image])?;
        Ok(embeddings.remove(0))
    }

    pub fn encode_batch(&mut self, images: &[&DynamicImage]) -> Result<Vec<Embedding>> {
        if images.is_empty() {
            return Err(Error::EmptyBatch);
        }
        let size = self.image_size as usize;
        let plane = size * size;
        let mut values = vec![0.0_f32; images.len() * plane * 3];
        for (batch_index, image) in images.iter().enumerate() {
            let image = prepare_image(image, self.image_size, self.resize)?;
            let batch_offset = batch_index * plane * 3;
            for (pixel_index, pixel) in image.pixels().enumerate() {
                for channel in 0..3 {
                    values[batch_offset + channel * plane + pixel_index] =
                        (f32::from(pixel[channel]) / 255.0 - self.mean[channel])
                            / self.std[channel];
                }
            }
        }
        let tensor = Tensor::from_array(([images.len(), 3_usize, size, size], values))?;
        let outputs = self.session.run(ort::inputs!["pixel_values" => tensor])?;
        embeddings_from_outputs(&outputs, images.len(), self.dimension)
    }
}

fn prepare_image(image: &DynamicImage, size: u32, resize: ImageResize) -> Result<RgbImage> {
    if image.width() == 0 || image.height() == 0 {
        return Err(Error::EmptyImage);
    }
    let rgb = image.to_rgb8();
    match resize {
        ImageResize::Stretch => Ok(image::imageops::resize(
            &rgb,
            size,
            size,
            FilterType::CatmullRom,
        )),
        ImageResize::ShortestEdgeCenterCrop => {
            let shortest = rgb.width().min(rgb.height());
            let resized_width = ((u64::from(rgb.width()) * u64::from(size)
                + u64::from(shortest) / 2)
                / u64::from(shortest)) as u32;
            let resized_height = ((u64::from(rgb.height()) * u64::from(size)
                + u64::from(shortest) / 2)
                / u64::from(shortest)) as u32;
            let resized = image::imageops::resize(
                &rgb,
                resized_width,
                resized_height,
                FilterType::CatmullRom,
            );
            let left = (resized_width - size) / 2;
            let top = (resized_height - size) / 2;
            Ok(image::imageops::crop_imm(&resized, left, top, size, size).to_image())
        }
    }
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, Rgb, RgbImage};

    use super::prepare_image;
    use crate::ImageResize;

    #[test]
    fn shortest_edge_resize_center_crops_to_model_dimensions() {
        let mut image = RgbImage::from_pixel(8, 4, Rgb([10, 20, 30]));
        image.put_pixel(0, 0, Rgb([255, 0, 0]));

        let prepared = prepare_image(
            &DynamicImage::ImageRgb8(image),
            4,
            ImageResize::ShortestEdgeCenterCrop,
        )
        .unwrap();

        assert_eq!(prepared.dimensions(), (4, 4));
        assert_eq!(prepared.get_pixel(2, 2).0, [10, 20, 30]);
    }
}
