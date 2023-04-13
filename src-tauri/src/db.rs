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
        "
      CREATE TABLE IF NOT EXISTS table_version(
        id INTEGER NOT NULL
          CONSTRAINT table_version_pk PRIMARY KEY,
        version_code INTEGER NOT NULL
      );",
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
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS meme (
                id         INTEGER
                    CONSTRAINT meme_pk
                        PRIMARY KEY AUTOINCREMENT,
                content    TEXT NOT NULL,
                extra_data TEXT,
                summary    TEXT NOT NULL,
                desc       TEXT,
                CONSTRAINT meme_pk2
                    UNIQUE (id, content, extra_data)
            );

            CREATE TABLE IF NOT EXISTS tag (
                id        INTEGER NOT NULL
                    CONSTRAINT tag_pk
                        PRIMARY KEY,
                namespace TEXT    NOT NULL,
                value     INTEGER NOT NULL,
                CONSTRAINT tag_pk2
                    UNIQUE (namespace, value)
            );

            CREATE TABLE IF NOT EXISTS meme_tag (
                tag_id  INTEGER NOT NULL
                    CONSTRAINT table_name_tag_id_fk
                        REFERENCES tag,
                meme_id INTEGER NOT NULL
                    CONSTRAINT table_name_meme_id_fk
                        REFERENCES meme,
                CONSTRAINT table_name_pk
                    PRIMARY KEY (tag_id, meme_id)
            );"
      )?;
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
