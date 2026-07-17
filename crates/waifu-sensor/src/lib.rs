#![deny(unsafe_op_in_unsafe_fn)]

mod assets;
mod bundle;
mod database;
mod error;
mod features;
mod model;
mod paths;
mod sensor;
mod sqlite_vec;
mod tagger;
pub mod training;

pub use assets::BuiltinAssets;
pub use bundle::{Bundle, BundleCharacter, BundleManifest};
pub use database::{DatabaseSync, WaifuDatabase};
pub use error::{Error, Result};
pub use features::{Feature, FeatureSchema};
pub use model::{ExecutionPolicy, ModelManager, ModelManifest, Provider};
pub use paths::PlatformPaths;
pub use sensor::{
    AddCharacterOptions, CharacterDraft, CharacterId, CharacterMatch, FeatureDifference,
    ReferenceDraft, WaifuSensor,
};
pub use tagger::{MlDanbooruTagger, Tagger};

pub use image;
pub use rusqlite;
