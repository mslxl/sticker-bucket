use std::{collections::HashMap, fs, fs::File, hash::Hash, io::{BufReader, Read}, path::{Path, PathBuf}};

use serde::{Serialize, Deserialize};
use crate::db::{DATABASE, DatabaseMapper};
use crate::handler::database::add_file_to_library;

#[derive(Serialize)]
pub struct Meme {
    id: u32,
    content: String,
    extra_data: String,
    summary: String,
    desc: Option<String>,
}

#[derive(Serialize, Deserialize)]
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
pub async fn open_image_and_interfere() -> Option<InterferMeme> {
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

pub fn add_file(file: String, delete_after_add: bool) -> String{
    let path = PathBuf::from(file);
    let sha256 = add_file_to_library(&path);
    if delete_after_add {
        fs::remove_file(path).unwrap();
    }
    sha256
}


#[tauri::command]
pub fn add_meme(file: String, extra_data: Option<String>, summary: String, desc: Option<String>,tags: Vec<Tag>, remove_after_add: bool){
    let mut binding = DATABASE.lock().unwrap();
    let transaction = binding.transaction().unwrap();
    let file_id = add_file(file, remove_after_add);
    let tag_id:Vec<i64> = tags.into_iter().map(|tag| DatabaseMapper::get_or_put_tag(&transaction, &tag.namespace, &tag.value)).collect();
    let meme_id = DatabaseMapper::add_meme(&transaction, file_id, extra_data, summary, desc, None);
    for tag in tag_id {
        DatabaseMapper::link_tag_and_meme(&transaction, tag, meme_id)
    }
    transaction.commit().unwrap();
}

fn interfer_summary<P: AsRef<Path>>(file: P) -> Option<String> {
    None
}

fn interfer_tags<P: AsRef<Path>>(file: P, summary: &Option<String>) -> Vec<Tag> {
    vec![]
}
