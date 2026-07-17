use std::path::PathBuf;

use thiserror::Error;

/// Errors produced by the waifu sensor library.
#[derive(Debug, Error)]
pub enum Error {
    #[error("SQLite operation failed: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("failed to decode JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("image operation failed: {0}")]
    Image(#[from] image::ImageError),

    #[cfg(feature = "onnx")]
    #[error("ONNX Runtime operation failed: {0}")]
    Onnx(#[from] ort::Error),

    #[cfg(feature = "download")]
    #[error("model download failed: {0}")]
    Download(#[from] Box<ureq::Error>),

    #[error("feature schema is invalid: {0}")]
    InvalidFeatureSchema(String),

    #[error("bundle is invalid: {0}")]
    InvalidBundle(String),

    #[error("database contains an incompatible waifu-sensor schema: {0}")]
    IncompatibleDatabase(String),

    #[error(
        "bundle feature schema {bundle_schema} is incompatible with database schema {database_schema}"
    )]
    IncompatibleBundle {
        bundle_schema: String,
        database_schema: String,
    },

    #[error("expected a {expected}-dimensional vector, received {actual} dimensions")]
    InvalidVectorDimension { expected: usize, actual: usize },

    #[error("feature `{0}` is not part of the active feature schema")]
    UnknownFeature(String),

    #[error("feature probability for `{feature}` must be finite and in [0, 1], got {value}")]
    InvalidFeatureProbability { feature: String, value: f32 },

    #[error("at least one reference image is required")]
    EmptyReferenceImages,

    #[error("character name must not be empty")]
    EmptyCharacterName,

    #[error("character {0} does not exist")]
    CharacterNotFound(uuid::Uuid),

    #[error("character `{0}` does not exist")]
    CharacterNameNotFound(String),

    #[error("character `{0}` already exists")]
    DuplicateCharacter(String),

    #[error("model file does not exist: {0}")]
    ModelMissing(PathBuf),

    #[error("model checksum mismatch for {path}: expected {expected}, got {actual}")]
    ModelChecksumMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },

    #[error("model output is invalid: {0}")]
    InvalidModelOutput(String),

    #[error("the operating system did not provide a {0} directory")]
    PlatformDirectoryUnavailable(&'static str),

    #[error("sqlite-vec initialization failed with SQLite status {0}")]
    SqliteVecInitialization(i32),
}

pub type Result<T> = std::result::Result<T, Error>;
