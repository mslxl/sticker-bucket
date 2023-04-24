use std::fs;
use std::sync::Mutex;
use std::{path::PathBuf, borrow::Cow};
use serde::{Deserialize, Serialize};
use directories::BaseDirs;
use rusqlite::Connection;
use once_cell::sync::Lazy;

use crate::db;
use crate::meme::interfer;

pub static DATABASE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let p = BaseDirs::new()
        .unwrap()
        .data_local_dir()
        .join("meme_management");
    if !p.exists() {
        fs::create_dir_all(&p).unwrap()
    }
    p
});

pub static DATABASE_FILE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let p = DATABASE_DIR.join("file");
    if !p.exists() {
        fs::create_dir_all(&p).unwrap()
    }
    p
});

static DATABASE: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let mut conn = Connection::open(DATABASE_DIR.join("index.db")).unwrap();
    db::handle_version(&mut conn).unwrap();
    Mutex::new(conn)
});


#[tauri::command]
pub fn get_table_version() -> u32 {
    db::query_table_version_code(&DATABASE.lock().unwrap())
}

#[tauri::command]
pub fn get_sqlite_version() -> String{
    format!("SQLite {}", rusqlite::version())
}

#[tauri::command]
pub fn get_data_dir() -> Cow<'static, str> {
    DATABASE_DIR.to_string_lossy()
}

#[tauri::command]
pub fn get_image_real_path(image_id: &str)-> PathBuf{
    DATABASE_FILE_DIR.join(image_id)
}



#[derive(Serialize)]
pub struct Meme {
    pub id: u32,
    pub content: String,
    pub extra_data: Option<String>,
    pub summary: String,
    pub desc: String,
}

#[derive(Serialize, Deserialize)]
pub struct Tag {
    pub namespace: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct InterferedMeme {
    path: PathBuf,
    summary: Option<String>,
    tags: Vec<Tag>,
}

#[tauri::command]
pub async fn open_image_and_interfere() -> Option<InterferedMeme> {
    let path = tauri::api::dialog::blocking::FileDialogBuilder::new()
        .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp", "gif"])
        .set_title("Add")
        .pick_file()?;

    let summary = interfer::interfer_summary(&path);
    let tags = interfer::interfer_tags(&path, &summary);
    Some(InterferedMeme {
        path,
        summary,
        tags,
    })
}


#[tauri::command]
pub fn add_meme(
    file: String,
    extra_data: Option<String>,
    summary: String,
    desc: Option<String>,
    tags: Vec<Tag>,
    remove_after_add: bool,
) {
    let mut binding = DATABASE.lock().unwrap();
    let transaction = binding.transaction().unwrap();
    let file_id = db::add_file(file, remove_after_add);
    let tag_id: Vec<i64> = tags
        .into_iter()
        .map(|tag| db::query_or_insert_tag(&transaction, &tag.namespace, &tag.value))
        .collect();
    let meme_id = db::insert_meme(&transaction, file_id, extra_data, summary, desc, None);
    for tag in tag_id {
        db::link_tag_meme(&transaction, tag, meme_id)
    }
    transaction.commit().unwrap();
}

#[tauri::command]
pub fn query_all_memes_by_page(page: i32) -> Vec<Meme> {
    let binding = DATABASE.lock().unwrap();
    db::query_all_memes_by_page(&binding, page)
}

#[tauri::command]
pub fn query_meme_by_id(id: i64) -> Meme {
    let binding = DATABASE.lock().unwrap();
    db::query_meme_by_id(&binding, id)
}

#[tauri::command]
pub fn query_tag_by_meme_id(id: i64) -> Vec<Tag> {
    let binding = DATABASE.lock().unwrap();
    db::query_all_meme_tag(&binding, id)
}

#[tauri::command]
pub fn query_namespace_with_prefix(prefix: &str) -> Vec<String> {
    let binding = DATABASE.lock().unwrap();
    db::query_tag_namespace_with_prefix(&binding, prefix)
}

#[tauri::command]
pub fn query_tag_value_with_prefix(namespace: &str, prefix: &str) -> Vec<String> {
    let binding = DATABASE.lock().unwrap();
    db::query_tag_value_with_prefix(&binding, namespace, prefix)
}