use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{Error, Result};

/// One optimized ML-Danbooru feature used for character retrieval.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Feature {
    pub tag: String,
    pub weight: f32,
}

/// The ordered feature contract produced by the offline optimizer.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FeatureSchema {
    pub id: String,
    pub model_id: String,
    pub threshold: f32,
    pub features: Vec<Feature>,
}

impl FeatureSchema {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let reader = BufReader::new(File::open(path)?);
        Self::from_reader(reader)
    }

    pub(crate) fn from_slice(bytes: &[u8]) -> Result<Self> {
        Self::from_reader(bytes)
    }

    fn from_reader(reader: impl std::io::Read) -> Result<Self> {
        let schema: Self = serde_json::from_reader(reader)?;
        schema.validate()?;
        Ok(schema)
    }

    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(Error::InvalidFeatureSchema(
                "schema id must not be empty".to_owned(),
            ));
        }
        if self.model_id.trim().is_empty() {
            return Err(Error::InvalidFeatureSchema(
                "model id must not be empty".to_owned(),
            ));
        }
        if !self.threshold.is_finite() || !(0.0..=1.0).contains(&self.threshold) {
            return Err(Error::InvalidFeatureSchema(format!(
                "threshold must be finite and in [0, 1], got {}",
                self.threshold
            )));
        }
        if self.features.is_empty() {
            return Err(Error::InvalidFeatureSchema(
                "at least one feature is required".to_owned(),
            ));
        }

        let mut seen = std::collections::HashSet::with_capacity(self.features.len());
        for feature in &self.features {
            if feature.tag.trim().is_empty() {
                return Err(Error::InvalidFeatureSchema(
                    "feature tags must not be empty".to_owned(),
                ));
            }
            if !seen.insert(feature.tag.as_str()) {
                return Err(Error::InvalidFeatureSchema(format!(
                    "duplicate feature tag `{}`",
                    feature.tag
                )));
            }
            if !feature.weight.is_finite() || !(0.0..=1.0).contains(&feature.weight) {
                return Err(Error::InvalidFeatureSchema(format!(
                    "weight for `{}` must be finite and in [0, 1], got {}",
                    feature.tag, feature.weight
                )));
            }
        }
        Ok(())
    }

    pub fn dimension(&self) -> usize {
        self.features.len()
    }

    pub fn digest(&self) -> Result<String> {
        let encoded = serde_json::to_vec(self)?;
        Ok(hex::encode(Sha256::digest(encoded)))
    }

    pub fn vector_from_scores(&self, scores: &HashMap<String, f32>) -> Result<Vec<f32>> {
        self.features
            .iter()
            .map(|feature| {
                let value = scores.get(&feature.tag).copied().unwrap_or(0.0);
                validate_probability(&feature.tag, value)?;
                Ok(value)
            })
            .collect()
    }

    pub fn weighted(&self, vector: &[f32]) -> Result<Vec<f32>> {
        self.validate_vector(vector)?;
        Ok(vector
            .iter()
            .zip(&self.features)
            .map(|(value, feature)| value * feature.weight)
            .collect())
    }

    pub fn validate_vector(&self, vector: &[f32]) -> Result<()> {
        if vector.len() != self.dimension() {
            return Err(Error::InvalidVectorDimension {
                expected: self.dimension(),
                actual: vector.len(),
            });
        }
        for (value, feature) in vector.iter().zip(&self.features) {
            validate_probability(&feature.tag, *value)?;
        }
        Ok(())
    }

    pub fn apply_overrides(
        &self,
        vector: &mut [f32],
        overrides: &HashMap<String, f32>,
    ) -> Result<()> {
        self.validate_vector(vector)?;
        for (tag, value) in overrides {
            validate_probability(tag, *value)?;
            let index = self
                .features
                .iter()
                .position(|feature| feature.tag == *tag)
                .ok_or_else(|| Error::UnknownFeature(tag.clone()))?;
            vector[index] = *value;
        }
        Ok(())
    }
}

fn validate_probability(feature: &str, value: f32) -> Result<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(Error::InvalidFeatureProbability {
            feature: feature.to_owned(),
            value,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Feature, FeatureSchema};
    use crate::Error;

    fn schema() -> FeatureSchema {
        FeatureSchema {
            id: "test-v1".to_owned(),
            model_id: "model-v1".to_owned(),
            threshold: 0.4,
            features: vec![
                Feature {
                    tag: "red_hair".to_owned(),
                    weight: 1.0,
                },
                Feature {
                    tag: "blue_eyes".to_owned(),
                    weight: 0.5,
                },
            ],
        }
    }

    #[test]
    fn converts_scores_in_schema_order_and_applies_weights() {
        let schema = schema();
        let scores = HashMap::from([("blue_eyes".to_owned(), 0.8), ("unused".to_owned(), 0.9)]);

        let vector = schema.vector_from_scores(&scores).unwrap();
        let weighted = schema.weighted(&vector).unwrap();

        assert_eq!(vector, vec![0.0, 0.8]);
        assert_eq!(weighted, vec![0.0, 0.4]);
    }

    #[test]
    fn rejects_unknown_override_instead_of_silently_ignoring_it() {
        let schema = schema();
        let mut vector = vec![0.2, 0.3];

        let error = schema
            .apply_overrides(&mut vector, &HashMap::from([("hat".to_owned(), 1.0)]))
            .unwrap_err();

        assert!(matches!(error, Error::UnknownFeature(tag) if tag == "hat"));
        assert_eq!(vector, vec![0.2, 0.3]);
    }
}
