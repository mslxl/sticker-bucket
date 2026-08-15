use std::path::{Path, PathBuf};

use crate::{Bundle, ModelManager, ModelManifest, Result};

const BUNDLE_MANIFEST: &[u8] = include_bytes!("../assets/bundles/v3/manifest.json");
const BUNDLE_FEATURES: &[u8] = include_bytes!("../assets/bundles/v3/features.json");
const BUNDLE_CHARACTERS: &[u8] = include_bytes!("../assets/bundles/v3/characters.json.xz");
const MODEL_MANIFEST: &[u8] = include_bytes!("../assets/models/ml-danbooru/manifest.json");
const MODEL_CLASSES: &[u8] = include_bytes!("../assets/models/ml-danbooru/classes.json");

/// Immutable metadata resources shipped with waifu-sensor.
pub struct BuiltinAssets;

impl BuiltinAssets {
    pub fn bundle() -> Result<Bundle> {
        Bundle::from_slices(BUNDLE_MANIFEST, BUNDLE_FEATURES, BUNDLE_CHARACTERS)
    }

    pub fn model_manifest() -> Result<ModelManifest> {
        ModelManifest::from_slice(MODEL_MANIFEST)
    }

    pub fn model_classes() -> Result<Vec<String>> {
        Ok(serde_json::from_slice(MODEL_CLASSES)?)
    }

    pub fn model_path() -> Result<PathBuf> {
        let manifest = Self::model_manifest()?;
        Ok(ModelManager::path_in(
            &manifest,
            Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/models/ml-danbooru"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::ModelManager;

    use super::BuiltinAssets;

    #[test]
    fn builtin_resources_are_mutually_compatible() {
        let bundle = BuiltinAssets::bundle().unwrap();
        let manifest = BuiltinAssets::model_manifest().unwrap();
        let classes = BuiltinAssets::model_classes().unwrap();

        assert_eq!(bundle.characters.len(), 7_443);
        assert_eq!(manifest.id, bundle.feature_schema.model_id);
        assert!(
            bundle
                .feature_schema
                .features
                .iter()
                .all(|feature| classes.contains(&feature.tag))
        );
    }

    #[test]
    fn builtin_model_path_matches_the_model_manifest() {
        let expected = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/models/ml-danbooru/ml_caformer_m36_dec-5-97527.onnx");
        let manifest = BuiltinAssets::model_manifest().unwrap();

        assert_eq!(BuiltinAssets::model_path().unwrap(), expected);
        assert_eq!(
            ModelManager::path_in(&manifest, expected.parent().unwrap()),
            expected
        );
    }
}
