use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use xz2::read::XzDecoder;

use crate::{Error, FeatureSchema, Result};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BundleManifest {
    pub bundle_id: Uuid,
    pub revision: u64,
    pub source: String,
    pub feature_schema: PathBuf,
    pub characters: PathBuf,
    pub characters_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BundleCharacter {
    pub name: String,
    pub prototypes: Vec<Vec<f32>>,
}

#[derive(Clone, Debug)]
pub struct Bundle {
    pub manifest: BundleManifest,
    pub feature_schema: FeatureSchema,
    pub characters: Vec<BundleCharacter>,
    pub content_digest: String,
}

impl Bundle {
    pub fn open(directory: impl AsRef<Path>) -> Result<Self> {
        let directory = directory.as_ref();
        let manifest_path = directory.join("manifest.json");
        let manifest: BundleManifest =
            serde_json::from_reader(BufReader::new(File::open(&manifest_path)?))?;
        let feature_schema = FeatureSchema::from_path(directory.join(&manifest.feature_schema))?;
        let character_path = directory.join(&manifest.characters);
        let encoded = read_all(&character_path)?;
        Self::from_parts(manifest, feature_schema, &encoded, &character_path)
    }

    pub(crate) fn from_slices(
        manifest_bytes: &[u8],
        feature_schema_bytes: &[u8],
        characters_bytes: &[u8],
    ) -> Result<Self> {
        let manifest: BundleManifest = serde_json::from_slice(manifest_bytes)?;
        let feature_schema = FeatureSchema::from_slice(feature_schema_bytes)?;
        let character_path = manifest.characters.clone();
        Self::from_parts(manifest, feature_schema, characters_bytes, &character_path)
    }

    fn from_parts(
        manifest: BundleManifest,
        feature_schema: FeatureSchema,
        encoded: &[u8],
        character_path: &Path,
    ) -> Result<Self> {
        let content_digest = hex::encode(Sha256::digest(encoded));
        if content_digest != manifest.characters_sha256 {
            return Err(Error::InvalidBundle(format!(
                "character data checksum mismatch for {}: expected {}, got {}",
                character_path.display(),
                manifest.characters_sha256,
                content_digest
            )));
        }

        let character_json: serde_json::Value =
            if character_path.extension().is_some_and(|x| x == "xz") {
                serde_json::from_reader(XzDecoder::new(encoded))?
            } else {
                serde_json::from_slice(encoded)?
            };
        let characters = decode_characters(character_json)?;

        let bundle = Self {
            manifest,
            feature_schema,
            characters,
            content_digest,
        };
        bundle.validate()?;
        Ok(bundle)
    }

    pub fn validate(&self) -> Result<()> {
        self.feature_schema.validate()?;
        if self.manifest.source.trim().is_empty() {
            return Err(Error::InvalidBundle(
                "manifest source must not be empty".to_owned(),
            ));
        }
        if self.characters.is_empty() {
            return Err(Error::InvalidBundle(
                "bundle must contain at least one character".to_owned(),
            ));
        }

        let mut names = std::collections::HashSet::with_capacity(self.characters.len());
        for character in &self.characters {
            if character.name.trim().is_empty() {
                return Err(Error::InvalidBundle(
                    "bundle character names must not be empty".to_owned(),
                ));
            }
            if !names.insert(character.name.as_str()) {
                return Err(Error::InvalidBundle(format!(
                    "duplicate bundle character `{}`",
                    character.name
                )));
            }
            if character.prototypes.is_empty() {
                return Err(Error::InvalidBundle(format!(
                    "bundle character `{}` has no prototypes",
                    character.name
                )));
            }
            for prototype in &character.prototypes {
                self.feature_schema.validate_vector(prototype)?;
            }
        }
        Ok(())
    }

    pub fn character_id(&self, name: &str) -> Uuid {
        Uuid::new_v5(&self.manifest.bundle_id, name.as_bytes())
    }
}

fn decode_characters(value: serde_json::Value) -> Result<Vec<BundleCharacter>> {
    if value.is_array() {
        return Ok(serde_json::from_value(value)?);
    }
    let legacy: std::collections::BTreeMap<String, Vec<f32>> = serde_json::from_value(value)
        .map_err(|error| {
            Error::InvalidBundle(format!(
                "character data must be a bundle array or upstream name-to-vector map: {error}"
            ))
        })?;
    Ok(legacy
        .into_iter()
        .map(|(name, vector)| BundleCharacter {
            name,
            prototypes: vec![vector],
        })
        .collect())
}

fn read_all(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?.read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    #[test]
    fn opens_the_shipped_v3_bundle_with_optimized_features() {
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/bundles/v3");

        let bundle = super::Bundle::open(directory).unwrap();

        assert_eq!(bundle.characters.len(), 7_443);
        assert_eq!(bundle.feature_schema.dimension(), 312);
        assert_eq!(bundle.feature_schema.features[0].tag, "animal_ears");
        assert_eq!(bundle.characters[0].prototypes[0].len(), 312);
    }
}
