use std::path::PathBuf;

use crate::library::{Sticky, StickyThumb};
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
pub async fn create_pic_sticky(
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

    let sticky_id = library::insert_pic_sticky_record(&mut transaction, &filename, &name, pkg_id, None, None, None)?;
    for tag in tags {
        let tag_id = library::get_or_insert_tag(&mut transaction, &tag.namespace, &tag.value)?;
        library::add_tag_to_sticky(&mut transaction, sticky_id, tag_id)?;
    }
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn create_text_sticky(
    state: tauri::State<'_, StickyDBState>,
    content: String,
    pkg: String,
    tags: Vec<Tag>,
) -> Result<(), String> {
    let mut guard = state.conn.lock().await;
    let mut transaction = guard.transaction().map_err(|e| e.to_string())?;
    let pkg_id = library::get_or_create_package(&mut transaction, &pkg)?;

    let sticky_id = library::insert_text_sticky_record(&mut transaction, &content, pkg_id)?;

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
    let guard = state.conn.lock().await;
    let conn: &Connection = &guard;
    library::search_package(conn, &keyword)
}

#[tauri::command]
pub async fn search_sticky(
    state: tauri::State<'_, StickyDBState>,
    stmt: String,
    page: i32,
) -> Result<Vec<StickyThumb>, String> {
    let guard = state.conn.lock().await;
    let conn: &Connection = &guard;
    library::search_sticky(&state, conn, &stmt, page-1)
}


#[tauri::command]
pub async fn count_search_sticky_page(
    state: tauri::State<'_, StickyDBState>,
    stmt: String,
) -> Result<i32, String> {
    let guard = state.conn.lock().await;
    let conn: &Connection = &guard;
    library::count_search_sticky_page(conn, &stmt)
}


#[tauri::command]
pub async fn search_tag_ns(
    state: tauri::State<'_, StickyDBState>,
    prefix: &str,
) -> Result<Vec<String>, String> {
    let guard = state.conn.lock().await;
    let conn: &Connection = &guard;
    library::search_tag_ns(conn, prefix)
}

#[tauri::command]
pub async fn search_tag_value(
    state: tauri::State<'_, StickyDBState>,
    ns: &str,
    value_prefix: &str,
) -> Result<Vec<String>, String> {
    let guard = state.conn.lock().await;
    let conn: &Connection = &guard;
    library::search_tag_value(conn, ns, value_prefix)
}

#[tauri::command]
pub async fn blacklist_path(
    state: tauri::State<'_, StickyDBState>,
    path: &str,
) -> Result<(), String> {
    let guard = state.conn.lock().await;
    let conn: &Connection = &guard;
    library::blacklist_path(conn, path)
}

#[tauri::command]
pub async fn is_path_blacklist(
    state: tauri::State<'_, StickyDBState>,
    path: &str,
) -> Result<bool, String> {
    let guard = state.conn.lock().await;
    let conn: &Connection = &guard;
    library::is_path_blacklist(conn, path)
}

#[tauri::command]
pub async fn get_sticky_by_id(
    state: tauri::State<'_, StickyDBState>,
    id: i64
) -> Result<Sticky, String> {
    let guard = state.conn.lock().await;
    let conn: &Connection = &guard;
    library::get_sticky_by_id(&state, conn, id)
}
