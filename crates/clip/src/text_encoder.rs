use std::path::Path;

use ort::{session::Session, value::Tensor};
use tokenizers::{Tokenizer, utils::truncation::TruncationParams};

use crate::{Embedding, Error, ExecutionPolicy, ModelManifest, Result, execution::session_builder};

pub struct TextEncoder {
    session: Session,
    tokenizer: Tokenizer,
    model_id: String,
    dimension: usize,
    context_length: usize,
    pad_token_id: u32,
}

impl std::fmt::Debug for TextEncoder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TextEncoder")
            .field("model_id", &self.model_id)
            .field("dimension", &self.dimension)
            .finish_non_exhaustive()
    }
}

impl TextEncoder {
    pub fn load(directory: impl AsRef<Path>, policy: ExecutionPolicy) -> Result<Self> {
        let directory = directory.as_ref();
        let manifest = ModelManifest::from_directory(directory)?;
        Self::load_with_manifest(directory, &manifest, policy)
    }

    pub(crate) fn load_with_manifest(
        directory: &Path,
        manifest: &ModelManifest,
        policy: ExecutionPolicy,
    ) -> Result<Self> {
        let mut tokenizer =
            Tokenizer::from_file(directory.join(&manifest.tokenizer)).map_err(Error::Tokenizer)?;
        tokenizer
            .with_truncation(Some(TruncationParams {
                max_length: manifest.context_length,
                ..TruncationParams::default()
            }))
            .map_err(Error::Tokenizer)?;
        let session =
            session_builder(&policy)?.commit_from_file(directory.join(&manifest.text_model))?;
        Ok(Self {
            session,
            tokenizer,
            model_id: manifest.id.clone(),
            dimension: manifest.embedding_dimension,
            context_length: manifest.context_length,
            pad_token_id: manifest.pad_token_id,
        })
    }

    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn encode(&mut self, text: &str) -> Result<Embedding> {
        let mut embeddings = self.encode_batch(&[text])?;
        Ok(embeddings.remove(0))
    }

    pub fn encode_batch(&mut self, texts: &[&str]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Err(Error::EmptyBatch);
        }
        let encodings = self
            .tokenizer
            .encode_batch(texts.to_vec(), true)
            .map_err(Error::Tokenizer)?;
        let sequence_length = encodings
            .iter()
            .map(|encoding| encoding.len())
            .max()
            .ok_or(Error::EmptyBatch)?;
        if sequence_length > self.context_length {
            return Err(Error::InvalidModelOutput(format!(
                "tokenizer produced {sequence_length} tokens, exceeding context length {}",
                self.context_length
            )));
        }

        let batch = encodings.len();
        let elements = batch * sequence_length;
        let mut input_ids = vec![i64::from(self.pad_token_id); elements];
        let mut attention_mask = vec![0_i64; elements];
        let mut token_type_ids = vec![0_i64; elements];
        for (batch_index, encoding) in encodings.iter().enumerate() {
            let offset = batch_index * sequence_length;
            for (token_index, ((input_id, attention), token_type)) in encoding
                .get_ids()
                .iter()
                .zip(encoding.get_attention_mask())
                .zip(encoding.get_type_ids())
                .enumerate()
            {
                input_ids[offset + token_index] = i64::from(*input_id);
                attention_mask[offset + token_index] = i64::from(*attention);
                token_type_ids[offset + token_index] = i64::from(*token_type);
            }
        }

        let input_ids = Tensor::from_array(([batch, sequence_length], input_ids))?;
        let attention_mask = Tensor::from_array(([batch, sequence_length], attention_mask))?;
        let token_type_ids = Tensor::from_array(([batch, sequence_length], token_type_ids))?;
        let outputs = self.session.run(ort::inputs![
            "input_ids" => input_ids,
            "attention_mask" => attention_mask,
            "token_type_ids" => token_type_ids,
        ])?;
        embeddings_from_outputs(&outputs, batch, self.dimension)
    }
}

pub(crate) fn embeddings_from_outputs(
    outputs: &ort::session::SessionOutputs<'_>,
    expected_batch: usize,
    expected_dimension: usize,
) -> Result<Vec<Embedding>> {
    let mut output_iter = outputs.iter();
    let (output_name, value) = output_iter.next().ok_or_else(|| {
        Error::InvalidModelOutput("model produced no embedding outputs".to_owned())
    })?;
    if output_iter.next().is_some() {
        return Err(Error::InvalidModelOutput(
            "model must produce exactly one embedding output".to_owned(),
        ));
    }
    let (shape, values) = value.try_extract_tensor::<f32>()?;
    let shape = shape
        .iter()
        .map(|dimension| usize::try_from(*dimension))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| {
            Error::InvalidModelOutput(format!(
                "invalid embedding shape for `{output_name}`: {shape:?}"
            ))
        })?;
    if shape != [expected_batch, expected_dimension] {
        return Err(Error::InvalidModelOutput(format!(
            "expected embedding output shape [{expected_batch}, {expected_dimension}], got {shape:?}"
        )));
    }
    values
        .chunks_exact(expected_dimension)
        .map(|values| Embedding::normalize(values.to_vec(), expected_dimension))
        .collect()
}
