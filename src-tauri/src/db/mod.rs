use std::sync::Mutex;

use once_cell::sync::Lazy;
use rusqlite::{Connection, Result};

static DATABASE_VERSION: u32 = 0;

pub static DATABASE: Lazy<Mutex<DatabaseMapper>> = Lazy::new(|| {
    let conn = Connection::open("index.db").unwrap();

    Mutex::new(DatabaseMapper::new(conn).unwrap())
});

fn handle_versin(conn: &Connection) -> Result<()> {
    conn.execute(
        include_str!("create_tableversion.sql"),
        (),
    )?;
    let version: Option<u32> = conn.query_row(
        "SELECT version_code FROM table_version WHERE id = 1;",
        (),
        |row| Ok(row.get(0)?),
    ).unwrap_or(None);
    if let Some(version) = version {
        // old database
        if version < DATABASE_VERSION {
            // upgrade
        }
    } else {
        // new database
        conn.execute(
            "INSERT INTO table_version(id, version_code) VALUES (?1, ?2);",
            (1, DATABASE_VERSION),
        )?;
        conn.execute_batch(include_str!("create_database.sql"))?;
    }
    Ok(())
}
pub struct DatabaseMapper {
    conn: Connection,
}
impl DatabaseMapper {
    pub fn new(conn: Connection) -> Result<Self> {
        handle_versin(&conn)?;
        Ok(DatabaseMapper { conn })
    }
    pub fn get_table_version_code(&self) -> u32 {
        self.conn
            .query_row(
                "SELECT version_code FROM table_version WHERE id = 1;",
                (),
                |row| Ok(row.get(0).unwrap()),
            )
            .unwrap()
    }
}
