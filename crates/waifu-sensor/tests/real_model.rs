#![cfg(feature = "onnx")]

use std::{num::NonZeroUsize, path::Path};

use waifu_sensor::{
    BuiltinAssets, Bundle, ExecutionPolicy, MlDanbooruTagger, ModelManager, ModelManifest,
    WaifuSensor, image, rusqlite::Connection,
};

#[test]
fn real_model_matches_the_upstream_top_three() {
    let manifest_directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/models/ml-danbooru");
    let manifest = ModelManifest::from_path(manifest_directory.join("manifest.json")).unwrap();
    let model_path = BuiltinAssets::model_path().unwrap();
    ModelManager::verify(&manifest, &model_path).unwrap();
    let bundle =
        Bundle::open(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/bundles/v3")).unwrap();
    let tagger = MlDanbooruTagger::load(
        model_path,
        manifest_directory.join(manifest.classes),
        bundle.feature_schema.clone(),
        ExecutionPolicy::Cpu,
    )
    .unwrap();
    let (mut sensor, _) =
        WaifuSensor::open(Connection::open_in_memory().unwrap(), &bundle, tagger).unwrap();
    let image =
        image::open(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fixtures/urusai.jpg"))
            .unwrap();

    let matches = sensor
        .predict(&image, NonZeroUsize::new(3).unwrap())
        .unwrap();

    assert_eq!(
        matches
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "momoi (blue archive)",
            "iijima yun",
            "midori (blue archive)"
        ]
    );
    assert!((matches[0].distance - 1.832_513_8).abs() < 0.05);
}
