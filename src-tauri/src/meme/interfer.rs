use std::path::Path;

use crate::handler::database::Tag;

pub fn interfer_summary<P: AsRef<Path>>(_file: P) -> Option<String> {
    None
}

pub fn interfer_tags<P: AsRef<Path>>(_file: P, _summary: &Option<String>) -> Vec<Tag> {
    vec![]
}