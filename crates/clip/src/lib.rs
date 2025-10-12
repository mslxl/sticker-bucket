#![forbid(unsafe_code)]

//! Local CLIP text and image embedding with bundled ONNX model assets.

mod embedding;
mod error;
mod execution;
mod image_encoder;
mod manifest;
mod model;
mod text_encoder;

pub use embedding::{Embedding, cosine_similarity};
pub use error::{Error, Result};
pub use execution::{ExecutionPolicy, Provider};
pub use image;
pub use image_encoder::ImageEncoder;
pub use manifest::{ImageResize, ModelManifest};
pub use model::{BuiltinModel, ClipModel};
pub use text_encoder::TextEncoder;
