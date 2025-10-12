use std::{fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImageResize {
    Stretch,
    ShortestEdgeCenterCrop,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ModelManifest {
    pub schema_version: u32,
    pub id: String,
    pub source: String,
    pub revision: String,
    pub license: Option<String>,
    pub embedding_dimension: usize,
    pub text_model: String,
    pub image_model: String,
    pub tokenizer: String,
    pub context_length: usize,
    pub pad_token_id: u32,
    pub image_size: u32,
    pub image_resize: ImageResize,
    pub image_mean: [f32; 3],
    pub image_std: [f32; 3],
}

impl ModelManifest {
    pub fn from_directory(directory: impl AsRef<Path>) -> Result<Self> {
        let directory = directory.as_ref();
        let manifest: Self =
            serde_json::from_reader(BufReader::new(File::open(directory.join("model.json"))?))?;
        manifest.validate(directory)?;
        Ok(manifest)
    }

    pub(crate) fn validate(&self, directory: &Path) -> Result<()> {
        if self.schema_version != 1 {
            return Err(Error::InvalidManifest(format!(
                "unsupported schema version {}; expected 1",
                self.schema_version
            )));
        }
        if self.id.trim().is_empty() {
            return Err(Error::InvalidManifest(
                "model id must not be empty".to_owned(),
            ));
        }
        if self.source.trim().is_empty() || self.revision.trim().is_empty() {
            return Err(Error::InvalidManifest(
                "model source and revision must not be empty".to_owned(),
            ));
        }
        if self.embedding_dimension == 0 || self.context_length == 0 || self.image_size == 0 {
            return Err(Error::InvalidManifest(
                "embedding dimension, context length, and image size must be positive".to_owned(),
            ));
        }
        for (name, file) in [
            ("text_model", &self.text_model),
            ("image_model", &self.image_model),
            ("tokenizer", &self.tokenizer),
        ] {
            let path = Path::new(file);
            if path.components().count() != 1 || file.trim().is_empty() {
                return Err(Error::InvalidManifest(format!(
                    "{name} must be a plain filename, got `{file}`"
                )));
            }
            let asset = directory.join(path);
            if !asset.is_file() {
                return Err(Error::MissingAsset(asset));
            }
        }
        for (name, values) in [
            ("image_mean", self.image_mean),
            ("image_std", self.image_std),
        ] {
            if values.iter().any(|value| !value.is_finite()) {
                return Err(Error::InvalidManifest(format!(
                    "{name} values must be finite"
                )));
            }
        }
        if self.image_std.iter().any(|value| *value <= 0.0) {
            return Err(Error::InvalidManifest(
                "image_std values must be positive".to_owned(),
            ));
        }
        Ok(())
    }
}
