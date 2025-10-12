use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{Error, Result};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModelManifest {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub sha256: String,
    pub classes: PathBuf,
}

impl ModelManifest {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_reader(BufReader::new(File::open(path)?))
    }

    pub(crate) fn from_slice(bytes: &[u8]) -> Result<Self> {
        Self::from_reader(bytes)
    }

    fn from_reader(reader: impl Read) -> Result<Self> {
        let manifest: Self = serde_json::from_reader(reader)?;
        manifest.validate()?;
        Ok(manifest)
    }

    fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(Error::InvalidBundle(
                "model id must not be empty".to_owned(),
            ));
        }
        if self.filename.trim().is_empty() {
            return Err(Error::InvalidBundle(
                "model filename must not be empty".to_owned(),
            ));
        }
        if self.url.trim().is_empty() {
            return Err(Error::InvalidBundle(
                "model URL must not be empty".to_owned(),
            ));
        }
        if self.sha256.len() != 64 || hex::decode(&self.sha256).is_err() {
            return Err(Error::InvalidBundle(format!(
                "model SHA-256 must contain 64 hexadecimal characters, got `{}`",
                self.sha256
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Provider {
    Cpu,
    CoreMl,
    DirectMl,
    Cuda,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ExecutionPolicy {
    #[default]
    Auto,
    Cpu,
    Prefer(Vec<Provider>),
    Require(Provider),
}

pub struct ModelManager;

impl ModelManager {
    pub fn path_in(manifest: &ModelManifest, directory: impl AsRef<Path>) -> PathBuf {
        directory.as_ref().join(&manifest.filename)
    }

    pub fn verify(manifest: &ModelManifest, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(Error::ModelMissing(path.to_owned()));
        }
        let actual = sha256_file(path)?;
        if actual != manifest.sha256 {
            return Err(Error::ModelChecksumMismatch {
                path: path.to_owned(),
                expected: manifest.sha256.clone(),
                actual,
            });
        }
        Ok(())
    }
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut reader = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(hex::encode(digest.finalize()))
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::{ModelManager, ModelManifest};

    #[test]
    fn verifies_the_exact_model_digest() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("model.onnx");
        std::fs::write(&path, b"model bytes").unwrap();
        let manifest = ModelManifest {
            id: "test".to_owned(),
            filename: "model.onnx".to_owned(),
            url: "https://example.invalid/model.onnx".to_owned(),
            sha256: hex::encode(Sha256::digest(b"model bytes")),
            classes: "classes.json".into(),
        };

        ModelManager::verify(&manifest, &path).unwrap();
    }
}
