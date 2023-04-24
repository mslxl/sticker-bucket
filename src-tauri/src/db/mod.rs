use std::{path::{Path, PathBuf}, fs};

use rusqlite::{Connection, OptionalExtension, Result};
use sha256::try_digest;

use crate::handler::database::{Meme, Tag, DATABASE_FILE_DIR};

static DATABASE_VERSION: u32 = 0;

/// Create database if not exists
/// Update database if table version < [DATABASE_VERSION]
pub fn handle_version(conn: &mut Connection) -> Result<()> {
    let transaction = conn.transaction().unwrap();
    transaction.execute(include_str!("create_tableversion.sql"), ())?;
    let version: Option<u32> = transaction
        .query_row(
            "SELECT version_code FROM table_version WHERE id = 1;",
            (),
            |row| Ok(row.get(0)?),
        )
        .unwrap_or(None);
    if let Some(version) = version {
        // old database
        if version < DATABASE_VERSION {
            // upgrade
        }
    } else {
        // new database
        transaction.execute(
            "INSERT INTO table_version(id, version_code) VALUES (?1, ?2);",
            (1, DATABASE_VERSION),
        )?;
        transaction.execute_batch(include_str!("create_database.sql"))?;
    }
    transaction.commit()
}

/// Get database code in file
pub fn query_table_version_code(conn: &Connection) -> u32 {
    conn.query_row(
        "SELECT version_code FROM table_version WHERE id = 1;",
        (),
        |row| Ok(row.get(0).unwrap()),
    )
    .unwrap()
}

/// Insert tag into database, or donothing if it already exists
pub fn query_or_insert_tag(conn: &Connection, namespace: &str, value: &str) -> i64 {
    conn.query_row(
        "SELECT id FROM tag WHERE namespace = ?1 AND value = ?2;",
        (namespace, value),
        |row| Ok(row.get(0).unwrap()),
    )
    .optional()
    .unwrap()
    .unwrap_or_else(|| {
        conn.execute(
            "INSERT INTO tag(namespace, value) VALUES (?1, ?2);",
            (namespace, value),
        )
        .unwrap();
        conn.last_insert_rowid()
    })
}

/// Add tag to meme
/// Tag info is store in other table
pub fn link_tag_meme(conn: &Connection, tag_id: i64, meme_id: i64) {
    conn.execute(
        "INSERT OR IGNORE INTO meme_tag(tag_id, meme_id) VALUES (?1, ?2) ",
        (tag_id, meme_id),
    )
    .unwrap();
}

/// Add meme basic info to database
pub fn insert_meme(
    conn: &Connection,
    file_id: String,
    extra_data: Option<String>,
    summary: String,
    description: Option<String>,
    thumbnail: Option<String>,
) -> i64 {
    conn.execute(
            "INSERT INTO meme(content, extra_data, summary, desc, thumbnail) VALUES (?1, ?2, ?3, ?4, ?5)",
            (file_id, extra_data, summary, description, thumbnail)).unwrap();
    conn.last_insert_rowid()
}

pub fn query_all_memes_by_page(conn: &Connection, page: i32) -> Vec<Meme> {
    let mut stmt = conn.prepare("SELECT id, content, extra_data, summary, desc FROM meme ORDER BY update_time DESC LIMIT 50 OFFSET ?1").unwrap();
    let iter = stmt
        .query_map([page], |row| {
            Ok(Meme {
                id: row.get(0).unwrap(),
                content: row.get(1).unwrap(),
                extra_data: row.get(2).ok(),
                summary: row.get(3).unwrap(),
                desc: row.get(4).unwrap(),
            })
        })
        .unwrap();
    iter.map(|v| v.unwrap()).collect()
}

pub fn query_meme_by_id(conn: &Connection, id: i64) -> Meme {
    conn.query_row(
        "SELECT id, content, extra_data, summary, desc FROM meme WHERE id = ?1",
        [id],
        |row| {
            Ok(Meme {
                id: row.get(0).unwrap(),
                content: row.get(1).unwrap(),
                extra_data: row.get(2).ok(),
                summary: row.get(3).unwrap(),
                desc: row.get(4).unwrap(),
            })
        },
    )
    .unwrap()
}

pub fn query_all_meme_tag(conn: &Connection, id: i64) -> Vec<Tag> {
    let mut stmt = conn.prepare("SELECT tag.namespace, tag.value FROM meme_tag LEFT JOIN tag on meme_tag.tag_id = tag.id WHERE meme_tag.meme_id = ?1").unwrap();
    let iter = stmt
        .query_map([id], |row| {
            Ok(Tag {
                namespace: row.get(0).unwrap(),
                value: row.get(1).unwrap(),
            })
        })
        .unwrap();

    iter.map(|x| x.unwrap()).collect()
}

pub fn query_tag_namespace_with_prefix(conn: &Connection, prefix: &str) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT namespace FROM tag WHERE namespace LIKE ?1")
        .unwrap();
    let iter = stmt
        .query_map([format!("{}%", prefix)], |row| Ok(row.get(0).unwrap()))
        .unwrap();
    iter.map(|v| v.unwrap()).collect()
}

pub fn query_tag_value_with_prefix(
    conn: &Connection,
    namespace: &str,
    prefix: &str,
) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT value FROM tag WHERE namespace = ?1 AND value LIKE ?2")
        .unwrap();
    let iter = stmt
        .query_map([namespace, &format!("{}%", prefix)], |row| {
            Ok(row.get(0).unwrap())
        })
        .unwrap();
    iter.map(|v| v.unwrap()).collect()
}

fn add_file_to_library<P: AsRef<Path>>(file: P) -> String {
    let sha256 = try_digest(file.as_ref()).unwrap();
    let target = DATABASE_FILE_DIR.join(&sha256);
    fs::copy(file, target).unwrap();
    sha256
}
pub fn add_file(file: String, delete_after_add: bool) -> String {
    let path = PathBuf::from(file);
    let sha256 = add_file_to_library(&path);
    if delete_after_add {
        fs::remove_file(path).unwrap();
    }
    sha256
}