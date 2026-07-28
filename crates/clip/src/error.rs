use std::path::PathBuf;

use thiserror::Error;

/// Errors returned by model loading and encoding operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to decode model manifest: {0}")]
    Json(#[from] serde_json::Error),

    #[error("ONNX Runtime operation failed: {0}")]
    Onnx(#[from] ort::Error),

    #[error("tokenization failed: {0}")]
    Tokenizer(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("model manifest is invalid: {0}")]
    InvalidManifest(String),

    #[error("model asset does not exist: {0}")]
    MissingAsset(PathBuf),

    #[error("model output is invalid: {0}")]
    InvalidModelOutput(String),

    #[error("input batch must not be empty")]
    EmptyBatch,

    #[error("image dimensions must be non-zero")]
    EmptyImage,

    #[error("embedding dimension mismatch: left has {left}, right has {right}")]
    EmbeddingDimensionMismatch { left: usize, right: usize },
}

pub type Result<T> = std::result::Result<T, Error>;
