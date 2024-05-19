use std::path::{Path, PathBuf};

use itertools::Itertools;
use log::info;
use rusqlite::{OptionalExtension, Transaction};
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

pub fn create_sticky(
    transaction: &mut Transaction,
    hash: &str,
    name: &str,
    package: i64,
    sensor_id: Option<&str>,
) -> Result<i64, String> {
    transaction
        .execute(
            "INSERT INTO sticky (hash, name, package, sensor_id) VALUES (?1, ?2, ?3, ?4)",
            (hash, name, package, sensor_id),
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

pub fn search_package(conn: &Transaction, keyword: &str) -> Result<Vec<String>, String> {
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct StickyThumb {
    path: PathBuf,
    name: String,
    width: Option<i64>,
    height: Option<i64>,
}

pub fn search_sticky(
    state: &StickyDBState,
    stmt: &str,
    page: i32,
) -> Result<Vec<StickyThumb>, String> {
    let conn_guard = state.conn.try_lock().map_err(|e| e.to_string())?;
    info!("Search with: {}", &stmt);
    let parsed_stmt = parse_serach(stmt)?;
    let sql = search::sql::build_search_sql_stmt(parsed_stmt, page, 50);
    info!("Statement: {}", &sql);
    let mut stmt = conn_guard.prepare(&sql).map_err(|e| e.to_string())?;
    let res = stmt
        .query_and_then::<_, anyhow::Error, _, _>([], |row| {
            Ok(StickyThumb {
                path: state.locate_path(&row.get::<&str, String>("hash")?),
                name: row.get("name")?,
                width: row.get("width").ok(),
                height: row.get("height").ok(),
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
