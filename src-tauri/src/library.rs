use std::path::PathBuf;

use rusqlite::{OptionalExtension, Transaction};
use sha2::{Digest, Sha256};

use crate::cmd::library::StickyDBState;

fn get_package_id(transaction: &mut Transaction, name: &str) -> Result<Option<i64>, String> {
    transaction
        .query_row_and_then("SELECT id FROM package where name = ?1", [name], |row| {
            row.get(0)
        })
        .optional()
        .map_err(|e| e.to_string())
}
fn create_package(
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

fn create_sticky(
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

fn create_tag(transaction: &mut Transaction, namespace: &str, value: &str) -> Result<i64, String> {
    transaction
        .execute(
            "INSERT INTO tag (namespace, value) VALUES (?1, ?2)",
            [namespace, value],
        )
        .map_err(|e| e.to_string())?;
    Ok(transaction.last_insert_rowid())
}

fn get_tag(
    transaction: &mut Transaction,
    namespace: &str,
    value: &str,
) -> Result<Option<i64>, String> {
    transaction
        .query_row_and_then(
            "SELECT id where namespace = ?1 AND valu = ?2",
            [namespace, value],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())
}

fn add_tag_to_sticky(
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
        .map(|_|())
}

fn remove_tag_from_sticky(transaction: &mut Transaction, sticky_id: i64, tag_id: i64)->Result<(), String>{
    transaction
        .execute(
            "DELETE FROM sticky_tag WHERE sticky = ?1 AND tag = ?2",
            [sticky_id, tag_id],
        )
        .map_err(|e| e.to_string())
        .map(|_|())
}

async fn cpy_to_library(
    state: &StickyDBState,
    file: PathBuf,
    with_ext: bool,
) -> Result<String, String> {
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
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    log::info!("Copy file {:?} to {:?}", &file, &dst);
    tokio::fs::copy(file, dst)
        .await
        .map_err(|e| e.to_string())?;

    Ok(hash)
}
