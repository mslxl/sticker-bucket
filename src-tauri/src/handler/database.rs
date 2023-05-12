use directories::BaseDirs;
use once_cell::sync::Lazy;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::{borrow::Cow, path::PathBuf};
use tokio::sync::Mutex;

use crate::db;
use crate::error::Error;

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
pub async fn get_table_version() -> Result<i64, Error> {
    let binding = &DATABASE.lock().await;
    db::query_table_version_code(&binding)
}

#[tauri::command]
pub fn get_sqlite_version() -> String {
    format!("SQLite {}", rusqlite::version())
}

#[tauri::command]
pub fn get_data_dir() -> Cow<'static, str> {
    DATABASE_DIR.to_string_lossy()
}

#[tauri::command]
pub fn get_image_real_path(basename: &str) -> PathBuf {
    DATABASE_FILE_DIR.join(basename)
}

#[derive(Serialize)]
pub struct Meme {
    pub id: u32,
    pub content: String,
    pub extra_data: Option<String>,
    pub summary: String,
    pub desc: String,
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Tag {
    pub namespace: String,
    pub value: String,
}

#[tauri::command]
pub async fn add_meme(
    file: String,
    extra_data: Option<String>,
    summary: String,
    desc: Option<String>,
    tags: Vec<Tag>,
    remove_after_add: bool,
) -> Result<bool, Error> {
    let mut binding = DATABASE.lock().await;
    let transaction = binding.transaction().unwrap();

    let file_id = db::add_file(file, remove_after_add)?;
    let mut tag_id = Vec::new();

    // insert tag to database
    for t in tags {
        let id = db::query_or_insert_tag(&transaction, &t.namespace, &t.value)?;
        tag_id.push(id);
    }

    // insert meme to database
    let meme_id = db::insert_meme(&transaction, file_id, extra_data, summary, desc, None)?;

    // link meme and tag
    for id in tag_id {
        db::link_tag_meme(&transaction, id, meme_id)?
    }

    transaction.commit()?;
    Ok(true)
}

/// 更新数据库，只有当字段不为 None 时才进行更新。如果为 None 则不做操作
#[tauri::command]
pub async fn update_meme(
    id: i64,
    extra_data: Option<String>,
    summary: Option<String>,
    desc: Option<String>,
    tags: Option<Vec<Tag>>,
) -> Result<bool, Error> {
    let mut binding = DATABASE.lock().await;
    let transaction = binding.transaction().unwrap();
    db::update_meme(&transaction, id, extra_data, summary, desc, None)?;
    if let Some(mut tags) = tags {
        let old_tags = db::query_all_meme_tag(&transaction, id)?;
        for t in old_tags.iter() {
            if !tags.contains(t) {
                let tag_id = db::query_tag_id(&transaction, &t.namespace, &t.value)?.unwrap(); // 数据库中的约束保证此处一定存在 tag_id
                db::unlink_tag_meme(&transaction, tag_id, id, true)?;
            } else {
                tags.swap_remove(tags.iter().position(|v| v == t).unwrap());
            }
        }

        for t in tags {
            let tag_id = db::query_or_insert_tag(&transaction, &t.namespace, &t.value)?;
            db::link_tag_meme(&transaction, tag_id, id)?;
        }
    }
    db::update_meme_edit_time(&transaction, id)?;

    transaction.commit()?;
    Ok(true)
}

#[tauri::command]
pub async fn search_memes_by_text(stmt: &str, page: i32) -> Result<Vec<Meme>, Error> {
    let binding = DATABASE.lock().await;
    Ok(db::search_meme_by_stmt(&binding, stmt, page)?)
}

#[tauri::command]
pub async fn query_meme_by_id(id: i64) -> Result<Meme, Error> {
    let binding = DATABASE.lock().await;
    Ok(db::query_meme_by_id(&binding, id)?)
}

#[tauri::command]
pub async fn query_tag_by_meme_id(id: i64) -> Result<Vec<Tag>, Error> {
    let binding = DATABASE.lock().await;
    Ok(db::query_all_meme_tag(&binding, id)?)
}

#[tauri::command]
pub async fn query_namespace_with_prefix(prefix: &str) -> Result<Vec<String>, Error> {
    let binding = DATABASE.lock().await;
    Ok(db::query_tag_namespace_with_prefix(&binding, prefix)?)
}

#[tauri::command]
pub async fn query_tag_value_with_prefix(
    namespace: &str,
    prefix: &str,
) -> Result<Vec<String>, Error> {
    let binding = DATABASE.lock().await;
    let res = db::query_tag_value_with_prefix(&binding, namespace, prefix)?;
    Ok(res)
}

#[tauri::command]
pub async fn query_tag_namespace_by_value_fuzzy(value: &str) -> Result<Vec<Tag>, Error> {
    let binding = DATABASE.lock().await;
    let res = db::query_tag_value_fuzzy(&binding, value)?;
    Ok(res)
}

#[tauri::command]
pub async fn query_count_memes() -> Result<i64, Error> {
    let binding = DATABASE.lock().await;
    db::query_count_memes(&binding)
}

#[tauri::command]
pub async fn query_count_tags() -> Result<i64, Error> {
    let binding = DATABASE.lock().await;
    db::query_count_tags(&binding)
}
