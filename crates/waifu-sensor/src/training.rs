//! Offline feature-weight optimization compatible with the upstream method.
//!
//! Training samples are tagged once with ML-Danbooru. Character centroids are
//! computed from the training split, then a sequential Bayesian optimizer searches
//! per-tag weights against a separate validation split.

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
    path::Path,
};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::{Error, Feature, FeatureSchema, Result};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaggedSample {
    pub character: String,
    pub scores: HashMap<String, f32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaggedDataset {
    pub samples: Vec<TaggedSample>,
}

impl TaggedDataset {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let dataset: Self = serde_json::from_reader(BufReader::new(File::open(path)?))?;
        dataset.validate()?;
        Ok(dataset)
    }

    pub fn validate(&self) -> Result<()> {
        if self.samples.is_empty() {
            return Err(Error::InvalidFeatureSchema(
                "tagged dataset must contain samples".to_owned(),
            ));
        }
        for sample in &self.samples {
            if sample.character.trim().is_empty() {
                return Err(Error::InvalidFeatureSchema(
                    "tagged sample character must not be empty".to_owned(),
                ));
            }
            for (tag, score) in &sample.scores {
                if tag.trim().is_empty() {
                    return Err(Error::InvalidFeatureSchema(
                        "tagged sample tags must not be empty".to_owned(),
                    ));
                }
                if !score.is_finite() || !(0.0..=1.0).contains(score) {
                    return Err(Error::InvalidFeatureProbability {
                        feature: tag.clone(),
                        value: *score,
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OptimizationConfig {
    pub schema_id: String,
    pub model_id: String,
    pub threshold: f32,
    pub candidate_limit: usize,
    pub iterations: usize,
    pub proposals_per_iteration: usize,
    pub minimum_weight: f32,
    pub seed: u64,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            schema_id: "waifu-sensor-optimized-v1".to_owned(),
            model_id: "ml_caformer_m36_dec-5-97527".to_owned(),
            threshold: 0.4,
            candidate_limit: 1024,
            iterations: 256,
            proposals_per_iteration: 64,
            minimum_weight: 0.01,
            seed: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Accuracy {
    pub top1: f64,
    pub top3: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OptimizationResult {
    pub schema: FeatureSchema,
    pub accuracy: Accuracy,
    pub candidate_count: usize,
    pub evaluations: usize,
}

pub fn optimize_features(
    training: &TaggedDataset,
    validation: &TaggedDataset,
    config: &OptimizationConfig,
) -> Result<OptimizationResult> {
    training.validate()?;
    validation.validate()?;
    validate_config(config)?;

    let candidates = select_candidates(training, config.candidate_limit);
    if candidates.is_empty() {
        return Err(Error::InvalidFeatureSchema(
            "training data produced no candidate tags".to_owned(),
        ));
    }
    let centroids = character_centroids(training, &candidates)?;
    let validation_characters: HashSet<_> = validation
        .samples
        .iter()
        .map(|sample| sample.character.as_str())
        .collect();
    for character in validation_characters {
        if !centroids.contains_key(character) {
            return Err(Error::InvalidFeatureSchema(format!(
                "validation character `{character}` has no training samples"
            )));
        }
    }

    let mut optimizer = TpeOptimizer::new(candidates.len(), config.seed);
    // Match the six probes in upstream `训练代码/找标签.py`: its original
    // appearance-focused block occupies the first 130 dimensions and its later
    // general tags occupy the remainder.
    for weights in upstream_initial_probes(candidates.len()) {
        let accuracy = evaluate(&centroids, validation, &candidates, &weights);
        optimizer.observe(weights, objective(accuracy));
    }

    for _ in 0..config.iterations {
        let weights = optimizer.propose(config.proposals_per_iteration);
        let accuracy = evaluate(&centroids, validation, &candidates, &weights);
        optimizer.observe(weights, objective(accuracy));
    }

    let mut best_weights = optimizer
        .best()
        .ok_or_else(|| {
            Error::InvalidFeatureSchema("optimizer produced no observations".to_owned())
        })?
        .0
        .to_vec();
    let mut best_accuracy = evaluate(&centroids, validation, &candidates, &best_weights);

    // Deterministic backward elimination turns near-zero/no-value dimensions into an
    // actual compact runtime feature set after the Bayesian search has found a basin.
    for index in 0..best_weights.len() {
        let previous = best_weights[index];
        best_weights[index] = 0.0;
        let reduced = evaluate(&centroids, validation, &candidates, &best_weights);
        if objective(reduced) + f64::EPSILON < objective(best_accuracy) {
            best_weights[index] = previous;
        } else {
            best_accuracy = reduced;
        }
    }

    let features: Vec<_> = candidates
        .into_iter()
        .zip(best_weights)
        .filter(|(_, weight)| *weight >= config.minimum_weight)
        .map(|(tag, weight)| Feature { tag, weight })
        .collect();
    if features.is_empty() {
        return Err(Error::InvalidFeatureSchema(
            "optimizer removed every candidate feature".to_owned(),
        ));
    }
    let schema = FeatureSchema {
        id: config.schema_id.clone(),
        model_id: config.model_id.clone(),
        threshold: config.threshold,
        features,
    };
    schema.validate()?;

    Ok(OptimizationResult {
        schema,
        accuracy: best_accuracy,
        candidate_count: optimizer.dimension,
        evaluations: optimizer.observations.len(),
    })
}

fn validate_config(config: &OptimizationConfig) -> Result<()> {
    if config.schema_id.trim().is_empty() || config.model_id.trim().is_empty() {
        return Err(Error::InvalidFeatureSchema(
            "optimizer schema and model ids must not be empty".to_owned(),
        ));
    }
    if config.candidate_limit == 0 || config.iterations == 0 || config.proposals_per_iteration == 0
    {
        return Err(Error::InvalidFeatureSchema(
            "optimizer limits and iteration counts must be positive".to_owned(),
        ));
    }
    if !config.minimum_weight.is_finite() || !(0.0..=1.0).contains(&config.minimum_weight) {
        return Err(Error::InvalidFeatureSchema(format!(
            "minimum weight must be finite and in [0, 1], got {}",
            config.minimum_weight
        )));
    }
    Ok(())
}

fn select_candidates(dataset: &TaggedDataset, limit: usize) -> Vec<String> {
    let mut frequencies = HashMap::<String, usize>::new();
    for sample in &dataset.samples {
        for (tag, score) in &sample.scores {
            if *score > 0.0 {
                *frequencies.entry(tag.clone()).or_default() += 1;
            }
        }
    }
    let mut tags: Vec<_> = frequencies.into_iter().collect();
    tags.sort_by(|(left_tag, left_count), (right_tag, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_tag.cmp(right_tag))
    });
    tags.truncate(limit);
    tags.into_iter().map(|(tag, _)| tag).collect()
}

fn character_centroids(
    dataset: &TaggedDataset,
    candidates: &[String],
) -> Result<HashMap<String, Vec<f32>>> {
    let mut sums = HashMap::<String, (Vec<f32>, usize)>::new();
    for sample in &dataset.samples {
        let entry = sums
            .entry(sample.character.clone())
            .or_insert_with(|| (vec![0.0; candidates.len()], 0));
        for (index, tag) in candidates.iter().enumerate() {
            entry.0[index] += sample.scores.get(tag).copied().unwrap_or(0.0);
        }
        entry.1 += 1;
    }
    sums.into_iter()
        .map(|(character, (mut sum, count))| {
            if count == 0 {
                return Err(Error::InvalidFeatureSchema(format!(
                    "character `{character}` has zero samples"
                )));
            }
            for value in &mut sum {
                *value /= count as f32;
            }
            Ok((character, sum))
        })
        .collect()
}

fn evaluate(
    centroids: &HashMap<String, Vec<f32>>,
    validation: &TaggedDataset,
    candidates: &[String],
    weights: &[f32],
) -> Accuracy {
    let mut top1 = 0_usize;
    let mut top3 = 0_usize;
    for sample in &validation.samples {
        let vector: Vec<_> = candidates
            .iter()
            .map(|tag| sample.scores.get(tag).copied().unwrap_or(0.0))
            .collect();
        let mut ranked: Vec<_> = centroids
            .iter()
            .map(|(character, centroid)| {
                let distance = vector
                    .iter()
                    .zip(centroid)
                    .zip(weights)
                    .map(|((value, mean), weight)| {
                        let difference = weight * (value - mean);
                        difference * difference
                    })
                    .sum::<f32>()
                    .sqrt();
                (character, distance)
            })
            .collect();
        ranked.sort_by(|(left_name, left_distance), (right_name, right_distance)| {
            left_distance
                .total_cmp(right_distance)
                .then_with(|| left_name.cmp(right_name))
        });
        if ranked
            .first()
            .is_some_and(|(character, _)| character.as_str() == sample.character)
        {
            top1 += 1;
        }
        if ranked
            .iter()
            .take(3)
            .any(|(character, _)| character.as_str() == sample.character)
        {
            top3 += 1;
        }
    }
    Accuracy {
        top1: top1 as f64 / validation.samples.len() as f64,
        top3: top3 as f64 / validation.samples.len() as f64,
    }
}

fn objective(accuracy: Accuracy) -> f64 {
    accuracy.top1 + accuracy.top3 * 1e-6
}

fn upstream_initial_probes(dimension: usize) -> Vec<Vec<f32>> {
    let split = dimension.min(130);
    let split_probe = |first: f32, rest: f32| {
        (0..dimension)
            .map(|index| if index < split { first } else { rest })
            .collect()
    };
    vec![
        split_probe(1.0, 0.0),
        split_probe(0.0, 1.0),
        split_probe(1.0, 0.5),
        split_probe(0.5, 1.0),
        vec![1.0; dimension],
        vec![0.5; dimension],
    ]
}

struct TpeOptimizer {
    dimension: usize,
    observations: Vec<(Vec<f32>, f64)>,
    random: ChaCha8Rng,
}

impl TpeOptimizer {
    fn new(dimension: usize, seed: u64) -> Self {
        Self {
            dimension,
            observations: Vec::new(),
            random: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    fn observe(&mut self, weights: Vec<f32>, score: f64) {
        debug_assert_eq!(weights.len(), self.dimension);
        self.observations.push((weights, score));
    }

    fn best(&self) -> Option<(&[f32], f64)> {
        self.observations
            .iter()
            .max_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(weights, score)| (weights.as_slice(), *score))
    }

    fn propose(&mut self, proposal_count: usize) -> Vec<f32> {
        if self.observations.len() < 8 {
            return (0..self.dimension)
                .map(|_| self.random.random_range(0.0..=1.0))
                .collect();
        }
        let mut ranked = self.observations.clone();
        ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
        let good_count = (ranked.len() / 5).max(2);
        let (good, bad) = ranked.split_at(good_count);

        let good_stats = distribution_stats(good, self.dimension);
        let bad_stats = distribution_stats(bad, self.dimension);
        let mut best_candidate = Vec::new();
        let mut best_ratio = f64::NEG_INFINITY;
        for _ in 0..proposal_count {
            let mut candidate = Vec::with_capacity(self.dimension);
            let mut ratio = 0.0_f64;
            for index in 0..self.dimension {
                let value =
                    truncated_normal(&mut self.random, good_stats[index].0, good_stats[index].1);
                ratio += log_normal_density(value, good_stats[index])
                    - log_normal_density(value, bad_stats[index]);
                candidate.push(value);
            }
            if ratio > best_ratio {
                best_ratio = ratio;
                best_candidate = candidate;
            }
        }
        best_candidate
    }
}

fn distribution_stats(observations: &[(Vec<f32>, f64)], dimension: usize) -> Vec<(f32, f32)> {
    (0..dimension)
        .map(|index| {
            let mean = observations
                .iter()
                .map(|(weights, _)| weights[index])
                .sum::<f32>()
                / observations.len() as f32;
            let variance = observations
                .iter()
                .map(|(weights, _)| {
                    let difference = weights[index] - mean;
                    difference * difference
                })
                .sum::<f32>()
                / observations.len() as f32;
            (mean, variance.sqrt().max(0.05))
        })
        .collect()
}

fn truncated_normal(random: &mut ChaCha8Rng, mean: f32, deviation: f32) -> f32 {
    // Box-Muller transform; clamping is the bounded [0, 1] prior used for weights.
    let first = random.random_range(f32::EPSILON..1.0);
    let second = random.random_range(0.0..1.0);
    let normal = (-2.0 * first.ln()).sqrt() * (std::f32::consts::TAU * second).cos();
    (mean + deviation * normal).clamp(0.0, 1.0)
}

fn log_normal_density(value: f32, distribution: (f32, f32)) -> f64 {
    let (mean, deviation) = distribution;
    let normalized = f64::from((value - mean) / deviation);
    -normalized * normalized / 2.0 - f64::from(deviation).ln()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{OptimizationConfig, TaggedDataset, TaggedSample, optimize_features};

    fn sample(character: &str, red: f32, blue: f32, nuisance: f32) -> TaggedSample {
        TaggedSample {
            character: character.to_owned(),
            scores: HashMap::from([
                ("red_hair".to_owned(), red),
                ("blue_hair".to_owned(), blue),
                ("smile".to_owned(), nuisance),
            ]),
        }
    }

    #[test]
    fn optimizes_tags_against_a_separate_validation_split() {
        let training = TaggedDataset {
            samples: vec![
                sample("red", 0.95, 0.05, 0.1),
                sample("red", 0.85, 0.05, 0.9),
                sample("blue", 0.05, 0.95, 0.1),
                sample("blue", 0.05, 0.85, 0.9),
            ],
        };
        let validation = TaggedDataset {
            samples: vec![
                sample("red", 0.9, 0.05, 0.95),
                sample("blue", 0.05, 0.9, 0.05),
            ],
        };
        let config = OptimizationConfig {
            iterations: 16,
            proposals_per_iteration: 8,
            minimum_weight: 0.01,
            ..OptimizationConfig::default()
        };

        let result = optimize_features(&training, &validation, &config).unwrap();

        assert_eq!(result.accuracy.top1, 1.0);
        assert_eq!(result.accuracy.top3, 1.0);
        assert!(
            result
                .schema
                .features
                .iter()
                .any(|feature| feature.tag == "red_hair" || feature.tag == "blue_hair")
        );
    }
}
