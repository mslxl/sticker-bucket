use std::path::Path;

use crate::{handler::database::Tag, bridge::text_recongize};

pub fn interfer_summary<P: AsRef<Path>>(file: P) -> Option<String> {
    let text = text_recongize(file.as_ref().to_str().unwrap())?;
    if text.trim().is_empty() {
        None
    }else{
        Some(text)
    }
}

pub fn interfer_tags<P: AsRef<Path>>(_file: P, _summary: &Option<String>) -> Vec<Tag> {
    vec![]
}