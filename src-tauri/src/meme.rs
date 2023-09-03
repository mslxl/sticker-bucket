use std::path::PathBuf;

use rusqlite::{Connection, Error, OptionalExtension};

use crate::{
    db::{self, search::build_search_sql, MemeDatabaseConnection, MemeDatabaseState},
    file::{compute_path, copy_to_storage, store_to_storage},
    AppDir,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Tag {
    key: String,
    value: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MemeToAdd {
    name: String,
    description: Option<String>,
    ty: String,
    content: String,
    fav: bool,
    tags: Vec<Tag>,
    pkg_id: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MemeQueried {
    id: i64,
    name: String,
    description: Option<String>,
    ty: String,
    hash: String,
    path: String,
    fav: bool,
    trash: bool,
    pkg_id: i64,
}

/// Query tag id
/// if tag is not exists, it will be inserted into database
fn make_tag(conn: &Connection, name: &str, value: &str) -> Result<i64, String> {
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM tag WHERE key = ?1 AND value = ?2",
            (name, value),
            |row| Ok(row.get("id").unwrap()),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if let Some(id) = id {
        return Ok(id);
    }

    conn.execute("INSERT INTO tag(key, value) VALUES (?1, ?2)", (name, value))
        .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

fn insert_meme_tag(conn: &Connection, meme_id: i64, tag_id: i64) -> Result<(), String> {
    conn.execute(
        "INSERT INTO meme_tag(meme_id, tag_id) VALUES (?1, ?2)",
        (meme_id, tag_id),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn add_meme_record(
    db_state: tauri::State<'_, MemeDatabaseState>,
    mut item: MemeToAdd,
) -> Result<(), String> {
    // let mut connection = db_state.state.as_ref().unwrap().conn.lock().await;
    let mut guard = db_state.state.lock().await;
    let state = &mut guard.as_mut().unwrap();

    let conn = state.conn.transaction().map_err(|e| e.to_string())?;

    let tag_id = item
        .tags
        .iter()
        .map(|t| make_tag(&conn, &t.key, &t.value))
        .collect::<Result<Vec<i64>, String>>()?;

    match item.ty.as_str() {
        "image" => {
            let new_content = copy_to_storage(&state.path, PathBuf::from(&item.content))
                .map_err(|e| e.to_string())?;
            item.content = new_content
        }
        "text" => {
            let new_content = store_to_storage(&state.path, item.content.as_bytes(), Some("txt"))
                .map_err(|e| e.to_string())?;
            item.content = new_content
        }
        _ => {
            unreachable!()
        }
    }

    conn.execute(
        "INSERT INTO meme(name, description, ty, hash, fav, pkg_id)
      VALUES(?1, ?2, ?3, ?4, ?5, ?6 ) ",
        (
            item.name,
            item.description,
            item.ty,
            item.content,
            item.fav,
            item.pkg_id,
        ),
    )
    .map_err(|e| e.to_string())?;

    let meme_id = conn.last_insert_rowid();
    for tid in tag_id {
        insert_meme_tag(&conn, meme_id, tid)?;
    }

    conn.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn search_meme(
    state: tauri::State<'_, MemeDatabaseState>,
    stmt: String,
    page: i64,
) -> Result<Vec<MemeQueried>, String> {
    let guard = state.state.lock().await;
    let state = guard.as_ref().unwrap();
    let mut sql_stmt = build_search_sql(&stmt).map_err(|e| e.to_string())?;
    sql_stmt.push_str("trash != 1 ");
    sql_stmt.push_str(&format!(
        "ORDER BY update_time DESC LIMIT 30 OFFSET {}",
        30 * page
    ));

    let mut query = state.conn.prepare(&sql_stmt).unwrap();
    let result = query
        .query_map([], |row| {
            let hash: String = row.get("hash").unwrap();
            Ok(MemeQueried {
                id: row.get("id").unwrap(),
                name: row.get("name").unwrap(),
                description: row.get("description").ok(),
                ty: row.get("ty").unwrap(),
                path: compute_path(&state.path, &hash)
                    .to_str()
                    .unwrap()
                    .to_owned(),
                hash: hash,
                fav: row.get("fav").unwrap(),
                trash: row.get("trash").unwrap(),
                pkg_id: row.get("pkg_id").unwrap(),
            })
        })
        .unwrap()
        .collect::<Result<Vec<MemeQueried>, Error>>()
        .map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
pub async fn get_meme_by_id(
    state: tauri::State<'_, MemeDatabaseState>,
    id: i64,
) -> Result<MemeQueried, String> {
    let guard = state.state.lock().await;
    let state = guard.as_ref().unwrap();
    let result = state
        .conn
        .query_row("SELECT * FROM meme WHERE id = ?1", [id], |row| {
            let hash: String = row.get("hash").unwrap();
            Ok(MemeQueried {
                id: row.get("id").unwrap(),
                name: row.get("name").unwrap(),
                description: row.get("description").ok(),
                ty: row.get("ty").unwrap(),
                path: compute_path(&state.path, &hash)
                    .to_str()
                    .unwrap()
                    .to_owned(),
                hash: hash,
                fav: row.get("fav").unwrap(),
                trash: row.get("trash").unwrap(),
                pkg_id: row.get("pkg_id").unwrap(),
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
pub async fn get_tags_by_id(
    state: tauri::State<'_, MemeDatabaseState>,
    id: i64,
) -> Result<Vec<Tag>, String> {
    let guard = state.state.lock().await;
    let state = guard.as_ref().unwrap();

    let mut query = state.conn.prepare("SELECT key, value FROM tag LEFT JOIN meme_tag ON tag.id = meme_tag.tag_id WHERE meme_tag.meme_id = ?1").unwrap();
    let result = query
        .query_map([id], |row| {
            Ok(Tag {
                key: row.get("key").unwrap(),
                value: row.get("value").unwrap(),
            })
        })
        .unwrap()
        .collect::<Result<Vec<Tag>, Error>>()
        .map_err(|e| e.to_string())?;

    Ok(result)
}
