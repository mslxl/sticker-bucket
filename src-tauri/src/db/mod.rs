use std::path::PathBuf;

use rusqlite::{Connection, OptionalExtension};
use tokio::sync::Mutex;

pub mod search;
pub struct MemeDatabaseConnection {
    pub path: PathBuf,
    pub conn: Connection,
}
pub struct MemeDatabaseState {
    pub state: Mutex<Option<MemeDatabaseConnection>>,
}

impl Default for MemeDatabaseState {
    fn default() -> Self {
        Self {
            state: Mutex::new(None),
        }
    }
}
impl MemeDatabaseState {
    async fn close(&self) {
        *self.state.lock().await = None;
    }

    async fn open(&self, path: PathBuf) {
        *self.state.lock().await = Some(MemeDatabaseConnection::open(path))
    }
}

impl MemeDatabaseConnection {
    const CURRENT_VERSION: u32 = 1;
    fn init(conn: &mut Connection) {
        let conn = conn.transaction().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS table_version (id INTEGER PRIMARY KEY, version INTEGER);",
            [],
        )
        .unwrap();

        let version: Option<u32> = conn
            .query_row(
                "SELECT version FROM table_version WHERE id = 0",
                (),
                |row| Ok(row.get(0)?),
            )
            .optional()
            .ok()
            .unwrap_or(None);

        if let Some(_local_version) = version {
            // upgrade local database
        } else {
            // create database
            conn.execute_batch(&format!(
                "INSERT OR REPLACE INTO table_version(id, version) VALUES(0, {});",
                Self::CURRENT_VERSION
            ))
            .unwrap();

            conn.execute_batch(include_str!("database_init.sql"))
                .unwrap();
        }
        conn.commit().unwrap();
    }

    pub fn open(path: PathBuf) -> Self {
        let mut conn = Connection::open(path.join("meme.db")).unwrap();

        Self::init(&mut conn);
        Self { path, conn }
    }
}

#[tauri::command]
pub async fn is_storage_available(
    state: tauri::State<'_, MemeDatabaseState>,
) -> Result<bool, String> {
    Ok(state.state.lock().await.is_some())
}

#[tauri::command]
pub async fn get_storage(state: tauri::State<'_, MemeDatabaseState>) -> Result<String, String> {
    let guard = state.state.lock().await;
    let path = guard.as_ref().unwrap().path.to_str().unwrap();
    Ok(path.to_owned())
}

#[tauri::command]
pub async fn open_storage(
    state: tauri::State<'_, MemeDatabaseState>,
    path: String,
) -> Result<(), String> {
    state.open(PathBuf::from(path)).await;
    Ok(())
}
