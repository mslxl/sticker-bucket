use std::path::{Path, PathBuf};

use image::DynamicImage;

use crate::{Embedding, ExecutionPolicy, ImageEncoder, ModelManifest, Result, TextEncoder};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuiltinModel {
    ChineseClipVitBasePatch16,
    TaiyiClipRoberta102mVitBasePatch32,
}

impl BuiltinModel {
    pub const fn directory_name(self) -> &'static str {
        match self {
            Self::ChineseClipVitBasePatch16 => "chinese-clip-vit-base-patch16",
            Self::TaiyiClipRoberta102mVitBasePatch32 => "taiyi-clip-roberta-102m-vit-base-patch32",
        }
    }

    pub fn directory(self) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/models")
            .join(self.directory_name())
    }
}

pub struct ClipModel {
    manifest: ModelManifest,
    text: TextEncoder,
    image: ImageEncoder,
}

impl std::fmt::Debug for ClipModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ClipModel")
            .field("model_id", &self.manifest.id)
            .field("dimension", &self.manifest.embedding_dimension)
            .finish_non_exhaustive()
    }
}

impl ClipModel {
    pub fn load(directory: impl AsRef<Path>, policy: ExecutionPolicy) -> Result<Self> {
        let directory = directory.as_ref();
        let manifest = ModelManifest::from_directory(directory)?;
        let text = TextEncoder::load_with_manifest(directory, &manifest, policy.clone())?;
        let image = ImageEncoder::load_with_manifest(directory, &manifest, policy)?;
        Ok(Self {
            manifest,
            text,
            image,
        })
    }

    pub fn load_builtin(model: BuiltinModel, policy: ExecutionPolicy) -> Result<Self> {
        Self::load(model.directory(), policy)
    }

    pub fn model_id(&self) -> &str {
        &self.manifest.id
    }

    pub fn dimension(&self) -> usize {
        self.manifest.embedding_dimension
    }

    pub fn text_encoder(&mut self) -> &mut TextEncoder {
        &mut self.text
    }

    pub fn image_encoder(&mut self) -> &mut ImageEncoder {
        &mut self.image
    }

    pub fn encode_text(&mut self, text: &str) -> Result<Embedding> {
        self.text.encode(text)
    }

    pub fn encode_texts(&mut self, texts: &[&str]) -> Result<Vec<Embedding>> {
        self.text.encode_batch(texts)
    }

    pub fn encode_image(&mut self, image: &DynamicImage) -> Result<Embedding> {
        self.image.encode(image)
    }

    pub fn encode_images(&mut self, images: &[&DynamicImage]) -> Result<Vec<Embedding>> {
        self.image.encode_batch(images)
    }
}
