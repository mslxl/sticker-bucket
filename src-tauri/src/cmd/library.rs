use std::path::PathBuf;

use crate::library::StickyThumb;
use anyhow::Result;
use rusqlite::Connection;
use tauri::{Manager, Runtime};
use tokio::sync::Mutex;

use crate::library::{self, Tag};

pub struct StickyDBState {
    path: PathBuf,
    pub conn: Mutex<Connection>,
}
impl StickyDBState {
    pub fn new(path: PathBuf) -> Result<StickyDBState> {
        let db_file = path.join("sticky.db");
        let is_new_db = !db_file.exists();
        let mut conn = Connection::open(db_file)?;
        if is_new_db {
            let trans = conn.transaction()?;
            trans.execute_batch(include_str!("library.sql"))?;
            trans.commit()?;
        }
        Ok(StickyDBState {
            path,
            conn: Mutex::new(conn),
        })
    }

    pub fn locate_path(&self, file_name: &str) -> PathBuf {
        let l1hash = &file_name[0..2];
        let l2hash = &file_name[2..4];
        self.path.join(l1hash).join(l2hash).join(file_name)
    }
}

#[tauri::command]
pub fn get_default_sticky_dir<R: Runtime>(app: tauri::AppHandle<R>) -> Result<String, String> {
    let data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let dir = data_dir.join("sticky");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(dir.to_str().unwrap().to_string())
}

#[tauri::command]
pub async fn has_sticky_file(
    state: tauri::State<'_, StickyDBState>,
    path: PathBuf,
    with_ext: bool,
) -> Result<bool, String> {
    let dst = library::predict_path(&state, &path, with_ext)?;
    Ok(dst.exists())
}

#[tauri::command]
pub async fn create_sticky(
    state: tauri::State<'_, StickyDBState>,
    name: String,
    pkg: String,
    path: PathBuf,
    tags: Vec<Tag>,
    with_ext: bool,
) -> Result<(), String> {
    let mut guard = state.conn.lock().await;
    let mut transaction = guard.transaction().map_err(|e| e.to_string())?;
    let filename = library::cpy_to_library(&state, path, with_ext)?;
    let pkg_id = library::get_or_create_package(&mut transaction, &pkg)?;

    let sticky_id = library::create_sticky(&mut transaction, &filename, &name, pkg_id, None)?;
    for tag in tags {
        let tag_id = library::get_or_insert_tag(&mut transaction, &tag.namespace, &tag.value)?;
        library::add_tag_to_sticky(&mut transaction, sticky_id, tag_id)?;
    }
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn search_package(
    state: tauri::State<'_, StickyDBState>,
    keyword: String,
) -> Result<Vec<String>, String> {
    let mut guard = state.conn.lock().await;
    let transaction = guard.transaction().map_err(|e| e.to_string())?;
    library::search_package(&transaction, &keyword)
}

#[tauri::command]
pub async fn search_sticky(
    state: tauri::State<'_, StickyDBState>,
    stmt: String,
    page: i32,
) -> Result<Vec<StickyThumb>, String> {
    library::search_sticky(&state, &stmt, page)
}
