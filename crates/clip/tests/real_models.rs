use memelith_clip::{
    BuiltinModel, ClipModel, ExecutionPolicy, cosine_similarity, image::ImageReader,
};

#[test]
#[ignore = "loads both bundled FP32 model pairs"]
fn bundled_models_encode_matching_text_and_image() {
    let image = ImageReader::open(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/cat.jpg"))
        .unwrap()
        .decode()
        .unwrap();

    for builtin in [
        BuiltinModel::ChineseClipVitBasePatch16,
        BuiltinModel::TaiyiClipRoberta102mVitBasePatch32,
    ] {
        let mut model = ClipModel::load_builtin(builtin, ExecutionPolicy::Cpu).unwrap();
        assert_eq!(model.dimension(), 512);

        let texts = model.encode_texts(&["一只猫", "一架飞机"]).unwrap();
        let images = model.encode_images(&[&image, &image]).unwrap();
        for embedding in texts.iter().chain(&images) {
            let norm = embedding
                .as_slice()
                .iter()
                .map(|value| value * value)
                .sum::<f32>()
                .sqrt();
            assert_eq!(embedding.dimension(), 512);
            assert!((norm - 1.0).abs() < 1e-5, "unexpected L2 norm {norm}");
        }

        let repeated_image_similarity = cosine_similarity(&images[0], &images[1]).unwrap();
        assert!(
            repeated_image_similarity > 0.9999,
            "{builtin:?} produced inconsistent image embeddings: {repeated_image_similarity}"
        );

        let cat_similarity = cosine_similarity(&texts[0], &images[0]).unwrap();
        let airplane_similarity = cosine_similarity(&texts[1], &images[0]).unwrap();
        assert!(
            cat_similarity > airplane_similarity,
            "{builtin:?} ranked airplane ({airplane_similarity}) above cat ({cat_similarity})"
        );
    }
}
