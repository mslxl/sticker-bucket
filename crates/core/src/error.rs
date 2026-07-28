use std::path::PathBuf;

use thiserror::Error;
use uuid::Uuid;

use crate::EmbeddingProviderError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("SQLite operation failed: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("I/O operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("image decoding failed: {0}")]
    Image(#[from] image::ImageError),

    #[error("embedding provider failed: {0}")]
    EmbeddingProvider(#[source] EmbeddingProviderError),

    #[error("storage root exists but is not a directory: {0}")]
    InvalidStorageRoot(PathBuf),

    #[error("embedding model id must not be empty")]
    EmptyEmbeddingModelId,

    #[error("embedding dimension must be positive")]
    EmptyEmbeddingDimension,

    #[error("database schema version {actual} is unsupported; expected {expected}")]
    UnsupportedSchemaVersion { expected: i64, actual: i64 },

    #[error(
        "database embedding space is incompatible: expected model `{expected_model}` with dimension {expected_dimension}, received `{actual_model}` with dimension {actual_dimension}"
    )]
    IncompatibleEmbeddingSpace {
        expected_model: String,
        expected_dimension: usize,
        actual_model: String,
        actual_dimension: usize,
    },

    #[error("{field} must not be empty")]
    EmptyField { field: &'static str },

    #[error("a Meme must contain at least one image or text block")]
    EmptyMemeContents,

    #[error("unsupported image format at {0}; expected PNG, JPEG, WebP, or GIF")]
    UnsupportedImageFormat(PathBuf),

    #[error("embedding for {field} has dimension {actual}; expected {expected}")]
    InvalidEmbeddingDimension {
        field: &'static str,
        expected: usize,
        actual: usize,
    },

    #[error("embedding for {field} contains a non-finite value at index {index}")]
    NonFiniteEmbedding { field: &'static str, index: usize },

    #[error("embedding for {field} has zero length")]
    ZeroEmbedding { field: &'static str },

    #[error("MemePack {0} does not exist")]
    MemePackNotFound(Uuid),

    #[error("Meme {0} does not exist")]
    MemeNotFound(Uuid),

    #[error("Tag {0} does not exist")]
    TagNotFound(Uuid),

    #[error("Tag `{0}` already exists")]
    DuplicateTag(String),

    #[error("Tag {tag_id} is already attached to {target} {target_id}")]
    TagAssociationExists {
        target: &'static str,
        target_id: Uuid,
        tag_id: Uuid,
    },

    #[error("Tag {tag_id} is not attached to {target} {target_id}")]
    TagAssociationNotFound {
        target: &'static str,
        target_id: Uuid,
        tag_id: Uuid,
    },

    #[error("database contains invalid data: {0}")]
    InvalidDatabase(String),

    #[error("managed media file is missing: {0}")]
    MissingMedia(PathBuf),

    #[error("managed media path is unsafe: {0}")]
    UnsafeMediaPath(PathBuf),
}

pub type Result<T> = std::result::Result<T, Error>;
