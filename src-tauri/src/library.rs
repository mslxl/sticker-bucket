use std::path::{Path, PathBuf};

use log::info;
use rusqlite::{Connection, OptionalExtension, Transaction};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
    cmd::library::StickyDBState,
    search::{self, parse_serach},
};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Tag {
    pub namespace: String,
    pub value: String,
}

pub fn get_package_id(transaction: &mut Transaction, name: &str) -> Result<Option<i64>, String> {
    transaction
        .query_row_and_then("SELECT id FROM package where name = ?1", [name], |row| {
            row.get(0)
        })
        .optional()
        .map_err(|e| e.to_string())
}
pub fn create_package(
    transaction: &mut Transaction,
    name: &str,
    description: Option<&str>,
    author: Option<&str>,
) -> Result<i64, String> {
    transaction
        .execute(
            "INSERT INTO package (name, description, author) VALUES (?1, ?2, ?3)",
            (name, description, author),
        )
        .map_err(|e| e.to_string())?;
    Ok(transaction.last_insert_rowid())
}

pub fn get_or_create_package(transaction: &mut Transaction, name: &str) -> Result<i64, String> {
    if let Some(id) = get_package_id(transaction, name)? {
        Ok(id)
    } else {
        create_package(transaction, name, None, None)
    }
}

pub fn insert_pic_sticky_record(
    transaction: &mut Transaction,
    filename: &str,
    name: &str,
    package: i64,
    width: Option<i64>,
    height: Option<i64>,
    sensor_id: Option<&str>,
) -> Result<i64, String> {
    transaction
        .execute(
            "INSERT INTO sticky (filename, name, package, sensor_id, width, height, type) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'STICKY')",
            (filename, name, package, sensor_id, width, height),
        )
        .map_err(|e| e.to_string())?;
    Ok(transaction.last_insert_rowid())
}

pub fn insert_text_sticky_record(
    transaction: &mut Transaction,
    content: &str,
    package: i64,
) -> Result<i64, String> {
    transaction
        .execute(
            "INSERT INTO sticky (name, package, type) VALUES (?1, ?2, 'TEXT')",
            (content, package),
        )
        .map_err(|e| e.to_string())?;
    Ok(transaction.last_insert_rowid())
}

pub fn create_tag(
    transaction: &mut Transaction,
    namespace: &str,
    value: &str,
) -> Result<i64, String> {
    transaction
        .execute(
            "INSERT INTO tag (namespace, value) VALUES (?1, ?2)",
            [namespace, value],
        )
        .map_err(|e| e.to_string())?;
    Ok(transaction.last_insert_rowid())
}

pub fn get_tag(
    transaction: &mut Transaction,
    namespace: &str,
    value: &str,
) -> Result<Option<i64>, String> {
    transaction
        .query_row_and_then(
            "SELECT id FROM tag where namespace = ?1 AND value = ?2",
            [namespace, value],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())
}

pub fn get_or_insert_tag(
    transaction: &mut Transaction,
    namespace: &str,
    value: &str,
) -> Result<i64, String> {
    if let Some(id) = get_tag(transaction, namespace, value)? {
        Ok(id)
    } else {
        create_tag(transaction, namespace, value)
    }
}

pub fn add_tag_to_sticky(
    transaction: &mut Transaction,
    sticky_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO sticky_tag (sticky, tag) VALUES (?1, ?2)",
            [sticky_id, tag_id],
        )
        .map_err(|e| e.to_string())
        .map(|_| ())
}

pub fn remove_tag_from_sticky(
    transaction: &mut Transaction,
    sticky_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    transaction
        .execute(
            "DELETE FROM sticky_tag WHERE sticky = ?1 AND tag = ?2",
            [sticky_id, tag_id],
        )
        .map_err(|e| e.to_string())
        .map(|_| ())
}

pub fn predict_path<P: AsRef<Path>>(
    state: &StickyDBState,
    file: P,
    with_ext: bool,
) -> Result<PathBuf, String> {
    let file = file.as_ref();
    let mut hasher = Sha256::new();
    let mut file_reader = std::fs::File::open(&file).map_err(|e| e.to_string())?;
    std::io::copy(&mut file_reader, &mut hasher).map_err(|e| e.to_string())?;
    let digest = hasher.finalize();
    let mut hash = format!("{:X}", digest);
    if with_ext {
        if let Some(ext) = file.extension() {
            hash.push('.');
            hash.push_str(&ext.to_string_lossy());
        }
    }

    let dst = state.locate_path(&hash);
    Ok(dst)
}

pub fn cpy_to_library(
    state: &StickyDBState,
    file: PathBuf,
    with_ext: bool,
) -> Result<String, String> {
    let dst = predict_path(state, &file, with_ext)?;
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    log::info!("Copy file {:?} to {:?}", &file, &dst);
    std::fs::copy(file, &dst).map_err(|e| e.to_string())?;

    Ok(dst.file_name().unwrap().to_str().unwrap().to_owned())
}

pub fn search_package(conn: &Connection, keyword: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT name FROM package WHERE name LIKE ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_and_then([format!("%{}%", keyword)], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let mut names: Vec<String> = Vec::new();

    for v in rows {
        names.push(v.map_err(|e| e.to_string())?);
    }
    Ok(names)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum StickTy {
    PIC,
    TEXT,
}

impl From<String> for StickTy {
    fn from(value: String) -> Self {
        match value.as_ref() {
            "TEXT" => StickTy::TEXT,
            _ => StickTy::PIC,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StickyThumb {
    id: i64,
    path: Option<PathBuf>,
    name: String,
    ty: StickTy,
    width: Option<i64>,
    height: Option<i64>,
}

pub fn search_sticky(
    state: &StickyDBState,
    conn: &Connection,
    stmt: &str,
    page: i32,
) -> Result<Vec<StickyThumb>, String> {
    info!("Search with: {}", &stmt);
    let parsed_stmt = parse_serach(stmt)?;
    let sql = search::sql::build_search_sql_stmt(parsed_stmt, page, 50);
    info!("Statement: {}", &sql);
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let res = stmt
        .query_and_then::<_, anyhow::Error, _, _>([], |row| {
            Ok(StickyThumb {
                id: row.get("id")?,
                path: row
                    .get::<&str, String>("filename")
                    .ok()
                    .map(|s| state.locate_path(&s)),
                name: row.get("name")?,
                width: row.get("width").ok(),
                height: row.get("height").ok(),
                ty: StickTy::from(row.get::<&str, String>("type").unwrap()),
            })
        })
        .optional()
        .map_err(|e| e.to_string())?;
    if let Some(value) = res {
        let mut stickies = Vec::new();
        for item in value {
            stickies.push(item.map_err(|e| e.to_string())?);
        }
        info!("Result number: {:?}", stickies.len());

        Ok(stickies)
    } else {
        info!("Result number: 0");
        Ok(Vec::new())
    }
}

pub fn search_tag_ns(conn: &Connection, prefix: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT(namespace) FROM tag WHERE namespace LIKE ?1 ORDER BY namespace")
        .map_err(|e| e.to_string())?;
    let items = stmt
        .query_and_then([format!("{}%", prefix)], |row| row.get::<usize, String>(0))
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(value) = items {
        let mut res = Vec::new();
        for i in value {
            res.push(i.map_err(|e| e.to_string())?);
        }
        Ok(res)
    } else {
        Ok(vec![])
    }
}

pub fn search_tag_value(
    conn: &Connection,
    ns: &str,
    value_prefix: &str,
) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT(value) FROM tag WHERE namespace = ?1 AND value LIKE ?2 ORDER BY namespace")
        .map_err(|e| e.to_string())?;
    let items = stmt
        .query_and_then((ns, format!("{}%", value_prefix)), |row| {
            row.get::<usize, String>(0)
        })
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(value) = items {
        let mut res = Vec::new();
        for i in value {
            res.push(i.map_err(|e| e.to_string())?);
        }
        Ok(res)
    } else {
        Ok(vec![])
    }
}

pub fn blacklist_path(conn: &Connection, path: &str) -> Result<(), String> {
    conn.execute("INSERT INTO blacklist_image (path) VALUES (?1)", [path])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn is_path_blacklist(conn: &Connection, path: &str) -> Result<bool, String> {
    let data = conn
        .query_row_and_then(
            "SELECT path FROM blacklist_image WHERE path = ?1",
            [path],
            |row| row.get::<usize, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(data.is_some())
}
