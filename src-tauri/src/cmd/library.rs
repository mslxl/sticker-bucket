use std::path::PathBuf;

use anyhow::Result;
use rusqlite::Connection;
use tauri::{Manager, Runtime};
use tokio::sync::Mutex;

pub struct StickyDBState {
    path: PathBuf,
    conn: Mutex<Connection>,
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

