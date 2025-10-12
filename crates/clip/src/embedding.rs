use crate::{Error, Result};

/// A finite, non-zero, L2-normalized vector in a CLIP embedding space.
#[derive(Clone, Debug, PartialEq)]
pub struct Embedding {
    values: Box<[f32]>,
}

impl Embedding {
    pub(crate) fn normalize(values: Vec<f32>, expected_dimension: usize) -> Result<Self> {
        if values.len() != expected_dimension {
            return Err(Error::InvalidModelOutput(format!(
                "expected {expected_dimension} embedding values, got {}",
                values.len()
            )));
        }
        if let Some((index, value)) = values
            .iter()
            .copied()
            .enumerate()
            .find(|(_, value)| !value.is_finite())
        {
            return Err(Error::InvalidModelOutput(format!(
                "embedding value {index} is not finite: {value}"
            )));
        }
        let squared_norm = values
            .iter()
            .map(|value| f64::from(*value) * f64::from(*value))
            .sum::<f64>();
        if !squared_norm.is_finite() || squared_norm <= f64::EPSILON {
            return Err(Error::InvalidModelOutput(
                "embedding has zero or non-finite L2 norm".to_owned(),
            ));
        }
        let norm = squared_norm.sqrt() as f32;
        Ok(Self {
            values: values
                .into_iter()
                .map(|value| value / norm)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        })
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.values
    }

    pub fn into_vec(self) -> Vec<f32> {
        self.values.into_vec()
    }

    pub fn dimension(&self) -> usize {
        self.values.len()
    }
}

/// Computes cosine similarity between normalized CLIP embeddings.
pub fn cosine_similarity(left: &Embedding, right: &Embedding) -> Result<f32> {
    if left.dimension() != right.dimension() {
        return Err(Error::EmbeddingDimensionMismatch {
            left: left.dimension(),
            right: right.dimension(),
        });
    }
    Ok(left
        .as_slice()
        .iter()
        .zip(right.as_slice())
        .map(|(left, right)| left * right)
        .sum())
}

#[cfg(test)]
mod tests {
    use super::{Embedding, cosine_similarity};
    use crate::Error;

    #[test]
    fn normalizes_vectors_and_computes_cosine_similarity() {
        let horizontal = Embedding::normalize(vec![3.0, 0.0], 2).unwrap();
        let diagonal = Embedding::normalize(vec![1.0, 1.0], 2).unwrap();

        assert_eq!(horizontal.as_slice(), &[1.0, 0.0]);
        let similarity = cosine_similarity(&horizontal, &diagonal).unwrap();
        assert!((similarity - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
    }

    #[test]
    fn rejects_zero_non_finite_and_wrong_dimension_vectors() {
        assert!(matches!(
            Embedding::normalize(vec![0.0, 0.0], 2),
            Err(Error::InvalidModelOutput(message)) if message.contains("zero")
        ));
        assert!(matches!(
            Embedding::normalize(vec![f32::NAN, 1.0], 2),
            Err(Error::InvalidModelOutput(message)) if message.contains("not finite")
        ));
        assert!(matches!(
            Embedding::normalize(vec![1.0], 2),
            Err(Error::InvalidModelOutput(message)) if message.contains("expected 2")
        ));
    }
}
