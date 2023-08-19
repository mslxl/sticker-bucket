use std::path::PathBuf;

use rusqlite::{Connection, OptionalExtension};
use tokio::sync::Mutex;

pub struct MemeDatabaseConnection {
    pub path: PathBuf,
    pub conn: Mutex<Connection>,
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
                MemeDatabaseConnection::CURRENT_VERSION
            ))
            .unwrap();

            conn.execute_batch(include_str!("database_init.sql"))
                .unwrap();
        }
        conn.commit().unwrap();
    }

    pub fn open(path: PathBuf) -> Self {
        let mut conn = Connection::open(path.join("meme.db")).unwrap();

        MemeDatabaseConnection::init(&mut conn);
        Self {
            path,
            conn: Mutex::new(conn),
        }
    }
}
