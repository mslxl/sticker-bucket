use std::{
    collections::HashMap,
    fs::File,
    hash::Hash,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use base64::Engine;
use serde::Serialize;
use serde_json::Serializer;

#[derive(Serialize)]
pub struct Meme {
    id: u32,
    content: String,
    extra_data: String,
    summary: String,
    desc: Option<String>,
}

#[derive(Serialize)]
pub struct Tag {
    namespace: String,
    value: String,
}

impl Tag {
    fn to_map(iter: impl IntoIterator<Item = Tag>) -> HashMap<String, Vec<String>> {
        iter.into_iter().fold(
            HashMap::new(),
            |mut prev: HashMap<String, Vec<String>>, x| {
                if let Some(vec) = prev.get_mut(&x.namespace) {
                    vec.push(x.value);
                } else {
                    let Tag { namespace, value } = x;
                    prev.insert(namespace, vec![value]);
                }
                prev
            },
        )
    }
    fn to_vec(map: HashMap<String, Vec<String>>) -> Vec<Tag> {
        map.into_iter()
            .map(|(namespace, values)| {
                values.into_iter().map(move |v| Tag {
                    namespace: namespace.clone(),
                    value: v,
                })
            })
            .flatten()
            .collect()
    }
}

#[derive(Serialize)]
pub struct InterferMeme {
    path: PathBuf,
    summary: Option<String>,
    tags: Vec<Tag>,
}

#[tauri::command]
pub async fn open_image_and_interfer() -> Option<InterferMeme> {
    let path = tauri::api::dialog::blocking::FileDialogBuilder::new()
        .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp"])
        .set_title("Add")
        .pick_file()?;

    let summary = interfer_summary(&path);
    let tags = interfer_tags(&path, &summary);
    Some(InterferMeme {
        path,
        summary,
        tags,
    })
}

fn interfer_summary<P: AsRef<Path>>(file: P) -> Option<String> {
    None
}

fn interfer_tags<P: AsRef<Path>>(file: P, summary: &Option<String>) -> Vec<Tag> {
    vec![]
}
