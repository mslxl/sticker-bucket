use std::{collections::HashMap, num::NonZeroUsize};

use image::DynamicImage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{DatabaseSync, Error, Result, Tagger, WaifuDatabase};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct CharacterId(Uuid);

impl CharacterId {
    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for CharacterId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::str::FromStr for CharacterId {
    type Err = uuid::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        value.parse().map(Self)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CharacterMatch {
    pub character_id: CharacterId,
    pub name: String,
    pub distance: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FeatureDifference {
    pub tag: String,
    pub query_probability: f32,
    pub target_probability: f32,
    pub weighted_delta: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AddCharacterOptions {
    pub prototype_merge_distance: f32,
}

impl Default for AddCharacterOptions {
    fn default() -> Self {
        Self {
            prototype_merge_distance: 0.25,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReferenceDraft {
    vector: Vec<f32>,
}

impl ReferenceDraft {
    pub fn probabilities(&self) -> &[f32] {
        &self.vector
    }
}

#[derive(Clone, Debug)]
pub struct CharacterDraft {
    name: String,
    feature_schema_id: String,
    references: Vec<ReferenceDraft>,
    options: AddCharacterOptions,
}

impl CharacterDraft {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn references(&self) -> &[ReferenceDraft] {
        &self.references
    }

    pub fn set_feature(
        &mut self,
        reference_index: usize,
        tag: &str,
        probability: f32,
        sensor: &WaifuSensor,
    ) -> Result<()> {
        let reference = self.references.get_mut(reference_index).ok_or_else(|| {
            Error::InvalidFeatureSchema(format!(
                "reference index {reference_index} is out of bounds"
            ))
        })?;
        sensor.database.feature_schema().apply_overrides(
            &mut reference.vector,
            &HashMap::from([(tag.to_owned(), probability)]),
        )
    }
}

pub struct WaifuSensor {
    database: WaifuDatabase,
    tagger: Box<dyn Tagger>,
}

impl std::fmt::Debug for WaifuSensor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WaifuSensor")
            .field("database", &self.database)
            .finish_non_exhaustive()
    }
}

impl WaifuSensor {
    pub fn new(database: WaifuDatabase, tagger: impl Tagger + 'static) -> Result<Self> {
        if database.feature_schema().id != tagger.feature_schema_id() {
            return Err(Error::InvalidFeatureSchema(format!(
                "database uses schema `{}`, but tagger uses `{}`",
                database.feature_schema().id,
                tagger.feature_schema_id()
            )));
        }
        Ok(Self {
            database,
            tagger: Box::new(tagger),
        })
    }

    pub fn open(
        connection: rusqlite::Connection,
        bundle: &crate::Bundle,
        tagger: impl Tagger + 'static,
    ) -> Result<(Self, DatabaseSync)> {
        let (database, sync) = WaifuDatabase::open(connection, bundle)?;
        Ok((Self::new(database, tagger)?, sync))
    }

    pub fn into_connection(self) -> rusqlite::Connection {
        self.database.into_connection()
    }

    /// Returns the optimized feature set used by both matching and character import.
    pub fn feature_schema(&self) -> &crate::FeatureSchema {
        self.database.feature_schema()
    }

    /// Resolves the stable internal identifier for an active character name.
    /// Name matching is case-insensitive, consistent with database uniqueness.
    pub fn character_id(&self, name: &str) -> Result<CharacterId> {
        self.database
            .character_id_by_name(name)
            .map(CharacterId::from_uuid)
    }

    pub fn predict(
        &mut self,
        image: &DynamicImage,
        top_n: NonZeroUsize,
    ) -> Result<Vec<CharacterMatch>> {
        let vector = self.tagger.tag(image)?;
        self.database.feature_schema().validate_vector(&vector)?;
        let weighted = self.database.feature_schema().weighted(&vector)?;
        self.database.search(&weighted, top_n).map(|matches| {
            matches
                .into_iter()
                .map(|item| CharacterMatch {
                    character_id: CharacterId(item.character_id),
                    name: item.name,
                    distance: item.distance,
                })
                .collect()
        })
    }

    pub fn why_not(
        &mut self,
        image: &DynamicImage,
        target: CharacterId,
        top_n: NonZeroUsize,
    ) -> Result<Vec<FeatureDifference>> {
        let query = self.tagger.tag(image)?;
        let schema = self.database.feature_schema();
        schema.validate_vector(&query)?;
        let prototypes = self.database.prototypes_for_character(target.0)?;
        let target_vector = prototypes
            .iter()
            .min_by(|left, right| {
                weighted_distance(schema, &query, left)
                    .total_cmp(&weighted_distance(schema, &query, right))
            })
            .ok_or(Error::CharacterNotFound(target.0))?;

        let mut differences: Vec<_> = schema
            .features
            .iter()
            .enumerate()
            .map(|(index, feature)| FeatureDifference {
                tag: feature.tag.clone(),
                query_probability: query[index],
                target_probability: target_vector[index],
                weighted_delta: feature.weight * (query[index] - target_vector[index]),
            })
            .collect();
        differences.sort_by(|left, right| {
            right
                .weighted_delta
                .abs()
                .total_cmp(&left.weighted_delta.abs())
        });
        differences.truncate(top_n.get());
        Ok(differences)
    }

    pub fn prepare_character(
        &mut self,
        name: impl Into<String>,
        images: &[DynamicImage],
        options: AddCharacterOptions,
    ) -> Result<CharacterDraft> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(Error::EmptyCharacterName);
        }
        if images.is_empty() {
            return Err(Error::EmptyReferenceImages);
        }
        validate_options(options)?;

        let references = images
            .iter()
            .map(|image| {
                let vector = self.tagger.tag(image)?;
                self.database.feature_schema().validate_vector(&vector)?;
                Ok(ReferenceDraft { vector })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(CharacterDraft {
            name,
            feature_schema_id: self.database.feature_schema().id.clone(),
            references,
            options,
        })
    }

    pub fn commit_character(&mut self, draft: CharacterDraft) -> Result<CharacterId> {
        if draft.feature_schema_id != self.database.feature_schema().id {
            return Err(Error::InvalidFeatureSchema(format!(
                "draft uses schema `{}`, active schema is `{}`",
                draft.feature_schema_id,
                self.database.feature_schema().id
            )));
        }
        let samples: Vec<_> = draft
            .references
            .into_iter()
            .map(|reference| reference.vector)
            .collect();
        let prototypes = cluster_prototypes(
            self.database.feature_schema(),
            &samples,
            draft.options.prototype_merge_distance,
        )?;
        self.database
            .insert_character(&draft.name, &samples, &prototypes)
            .map(CharacterId)
    }

    pub fn add_character(
        &mut self,
        name: impl Into<String>,
        images: &[DynamicImage],
        options: AddCharacterOptions,
    ) -> Result<CharacterId> {
        let draft = self.prepare_character(name, images, options)?;
        self.commit_character(draft)
    }

    pub fn add_character_images(
        &mut self,
        character_id: CharacterId,
        images: &[DynamicImage],
        options: AddCharacterOptions,
    ) -> Result<()> {
        if images.is_empty() {
            return Err(Error::EmptyReferenceImages);
        }
        validate_options(options)?;
        let new_samples = images
            .iter()
            .map(|image| {
                let vector = self.tagger.tag(image)?;
                self.database.feature_schema().validate_vector(&vector)?;
                Ok(vector)
            })
            .collect::<Result<Vec<_>>>()?;
        let mut all_samples = self.database.samples_for_character(character_id.0)?;
        all_samples.extend(new_samples.iter().cloned());
        let prototypes = cluster_prototypes(
            self.database.feature_schema(),
            &all_samples,
            options.prototype_merge_distance,
        )?;
        self.database
            .append_character_vectors(character_id.0, &new_samples, &prototypes)
    }
}

fn validate_options(options: AddCharacterOptions) -> Result<()> {
    if !options.prototype_merge_distance.is_finite() || options.prototype_merge_distance < 0.0 {
        return Err(Error::InvalidFeatureSchema(format!(
            "prototype merge distance must be finite and non-negative, got {}",
            options.prototype_merge_distance
        )));
    }
    Ok(())
}

fn cluster_prototypes(
    schema: &crate::FeatureSchema,
    samples: &[Vec<f32>],
    threshold: f32,
) -> Result<Vec<Vec<f32>>> {
    if samples.is_empty() {
        return Err(Error::EmptyReferenceImages);
    }
    for sample in samples {
        schema.validate_vector(sample)?;
    }
    let normalizer = schema
        .features
        .iter()
        .map(|feature| feature.weight * feature.weight)
        .sum::<f32>()
        .sqrt();
    if normalizer == 0.0 {
        return Err(Error::InvalidFeatureSchema(
            "at least one optimized feature weight must be positive".to_owned(),
        ));
    }

    let mut clusters: Vec<Vec<usize>> = (0..samples.len()).map(|index| vec![index]).collect();
    loop {
        let mut best: Option<(usize, usize, f32)> = None;
        for left in 0..clusters.len() {
            for right in (left + 1)..clusters.len() {
                let distance =
                    average_link_distance(schema, samples, &clusters[left], &clusters[right])
                        / normalizer;
                if distance <= threshold
                    && best.is_none_or(|(_, _, best_distance)| distance < best_distance)
                {
                    best = Some((left, right, distance));
                }
            }
        }
        let Some((left, right, _)) = best else {
            break;
        };
        let right_cluster = clusters.remove(right);
        clusters[left].extend(right_cluster);
        clusters[left].sort_unstable();
    }

    clusters
        .iter()
        .map(|cluster| {
            let mut centroid = vec![0.0; schema.dimension()];
            for sample_index in cluster {
                for (target, value) in centroid.iter_mut().zip(&samples[*sample_index]) {
                    *target += value;
                }
            }
            let count = cluster.len() as f32;
            for value in &mut centroid {
                *value /= count;
            }
            schema.validate_vector(&centroid)?;
            Ok(centroid)
        })
        .collect()
}

fn average_link_distance(
    schema: &crate::FeatureSchema,
    samples: &[Vec<f32>],
    left: &[usize],
    right: &[usize],
) -> f32 {
    let mut total = 0.0;
    for left_index in left {
        for right_index in right {
            total += weighted_distance(schema, &samples[*left_index], &samples[*right_index]);
        }
    }
    total / (left.len() * right.len()) as f32
}

fn weighted_distance(schema: &crate::FeatureSchema, left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right)
        .zip(&schema.features)
        .map(|((left, right), feature)| {
            let difference = feature.weight * (left - right);
            difference * difference
        })
        .sum::<f32>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use image::{DynamicImage, Rgb, RgbImage};
    use rusqlite::Connection;
    use uuid::Uuid;

    use crate::{
        Bundle, BundleCharacter, BundleManifest, Feature, FeatureSchema, Tagger, WaifuDatabase,
    };

    struct PixelTagger {
        schema_id: String,
    }

    impl Tagger for PixelTagger {
        fn feature_schema_id(&self) -> &str {
            &self.schema_id
        }

        fn tag(&mut self, image: &DynamicImage) -> crate::Result<Vec<f32>> {
            let pixel = image.to_rgb8().get_pixel(0, 0).0;
            Ok(vec![
                f32::from(pixel[0]) / 255.0,
                f32::from(pixel[2]) / 255.0,
            ])
        }
    }

    fn test_schema() -> FeatureSchema {
        FeatureSchema {
            id: "test".to_owned(),
            model_id: "model".to_owned(),
            threshold: 0.4,
            features: vec![
                Feature {
                    tag: "red".to_owned(),
                    weight: 1.0,
                },
                Feature {
                    tag: "blue".to_owned(),
                    weight: 1.0,
                },
            ],
        }
    }

    fn image(red: u8, blue: u8) -> DynamicImage {
        DynamicImage::ImageRgb8(RgbImage::from_pixel(1, 1, Rgb([red, 0, blue])))
    }

    fn test_sensor() -> super::WaifuSensor {
        let schema = test_schema();
        let bundle = Bundle {
            manifest: BundleManifest {
                bundle_id: Uuid::parse_str("211974f5-0bda-43a4-bd56-c2b13f4e2978").unwrap(),
                revision: 1,
                source: "test".to_owned(),
                feature_schema: "features.json".into(),
                characters: "characters.json".into(),
                characters_sha256: "unused".to_owned(),
            },
            feature_schema: schema.clone(),
            characters: vec![BundleCharacter {
                name: "baseline".to_owned(),
                prototypes: vec![vec![0.5, 0.5]],
            }],
            content_digest: "test".to_owned(),
        };
        let (database, _) =
            WaifuDatabase::open(Connection::open_in_memory().unwrap(), &bundle).unwrap();
        super::WaifuSensor::new(
            database,
            PixelTagger {
                schema_id: schema.id,
            },
        )
        .unwrap()
    }

    #[test]
    fn clusters_nearby_samples_but_keeps_distant_internal_prototypes() {
        let schema = FeatureSchema {
            id: "test".to_owned(),
            model_id: "model".to_owned(),
            threshold: 0.4,
            features: vec![
                Feature {
                    tag: "a".to_owned(),
                    weight: 1.0,
                },
                Feature {
                    tag: "b".to_owned(),
                    weight: 1.0,
                },
            ],
        };
        let samples = vec![vec![0.9, 0.1], vec![0.8, 0.1], vec![0.1, 0.9]];

        let prototypes = super::cluster_prototypes(&schema, &samples, 0.2).unwrap();

        assert_eq!(prototypes, vec![vec![0.85, 0.1], vec![0.1, 0.9]]);
    }

    #[test]
    fn add_character_is_direct_and_hides_multiple_internal_prototypes() {
        let mut sensor = test_sensor();
        let red = image(255, 0);
        let blue = image(0, 255);

        let character = sensor
            .add_character(
                "new character",
                &[red.clone(), blue.clone()],
                super::AddCharacterOptions {
                    prototype_merge_distance: 0.2,
                },
            )
            .unwrap();

        assert_eq!(
            sensor
                .database
                .prototypes_for_character(character.as_uuid())
                .unwrap(),
            vec![vec![1.0, 0.0], vec![0.0, 1.0]]
        );
        let red_matches = sensor.predict(&red, NonZeroUsize::new(2).unwrap()).unwrap();
        let blue_matches = sensor
            .predict(&blue, NonZeroUsize::new(2).unwrap())
            .unwrap();
        assert_eq!(red_matches[0].name, "new character");
        assert_eq!(blue_matches[0].name, "new character");
        assert_eq!(
            red_matches
                .iter()
                .filter(|item| item.character_id == character)
                .count(),
            1
        );
    }

    #[test]
    fn add_character_images_rebuilds_private_prototypes_from_all_samples() {
        let mut sensor = test_sensor();
        let red = image(255, 0);
        let blue = image(0, 255);
        let character = sensor
            .add_character(
                "expandable",
                std::slice::from_ref(&red),
                super::AddCharacterOptions {
                    prototype_merge_distance: 0.2,
                },
            )
            .unwrap();

        sensor
            .add_character_images(
                character,
                std::slice::from_ref(&blue),
                super::AddCharacterOptions {
                    prototype_merge_distance: 0.2,
                },
            )
            .unwrap();

        assert_eq!(
            sensor
                .database
                .prototypes_for_character(character.as_uuid())
                .unwrap(),
            vec![vec![1.0, 0.0], vec![0.0, 1.0]]
        );
    }

    #[test]
    fn resolves_a_character_name_case_insensitively() {
        let sensor = test_sensor();

        let character = sensor.character_id("BASELINE").unwrap();
        let missing = sensor.character_id("missing character").unwrap_err();

        assert_eq!(
            character,
            super::CharacterId::from_uuid(Uuid::new_v5(
                &Uuid::parse_str("211974f5-0bda-43a4-bd56-c2b13f4e2978").unwrap(),
                b"baseline"
            ))
        );
        assert!(matches!(
            missing,
            crate::Error::CharacterNameNotFound(name) if name == "missing character"
        ));
    }
}
