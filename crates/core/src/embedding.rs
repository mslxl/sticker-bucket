use std::error::Error as StdError;

use image::DynamicImage;

pub type EmbeddingProviderError = Box<dyn StdError + Send + Sync + 'static>;

/// Supplies embeddings in one stable vector space for a Memelith database.
pub trait EmbeddingProvider {
    fn model_id(&self) -> &str;
    fn dimension(&self) -> usize;
    fn embed_text(&mut self, text: &str) -> std::result::Result<Vec<f32>, EmbeddingProviderError>;
    fn embed_image(
        &mut self,
        image: &DynamicImage,
    ) -> std::result::Result<Vec<f32>, EmbeddingProviderError>;
}

impl EmbeddingProvider for memelith_clip::ClipModel {
    fn model_id(&self) -> &str {
        self.model_id()
    }

    fn dimension(&self) -> usize {
        self.dimension()
    }

    fn embed_text(&mut self, text: &str) -> std::result::Result<Vec<f32>, EmbeddingProviderError> {
        self.encode_text(text)
            .map(memelith_clip::Embedding::into_vec)
            .map_err(|error| Box::new(error) as EmbeddingProviderError)
    }

    fn embed_image(
        &mut self,
        image: &DynamicImage,
    ) -> std::result::Result<Vec<f32>, EmbeddingProviderError> {
        self.encode_image(image)
            .map(memelith_clip::Embedding::into_vec)
            .map_err(|error| Box::new(error) as EmbeddingProviderError)
    }
}
