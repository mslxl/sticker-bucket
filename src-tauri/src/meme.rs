use std::{collections::HashMap, path::PathBuf};

use rusqlite::{Connection, Error, OptionalExtension};

use crate::{
    db::{self, search::build_search_sql, MemeDatabaseConnection, MemeDatabaseState},
    file::{compute_path, copy_to_storage, store_to_storage},
    AppDir,
};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
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
pub async fn update_meme_record(
    db_state: tauri::State<'_, MemeDatabaseState>,
    meme_id: i64,
    item: MemeToAdd,
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

    conn.execute("UPDATE meme SET name = ?1, description = ?2, fav = ?3, pkg_id = ?4 WHERE id = ?5", (
        item.name,
        item.description,
        item.fav,
        item.pkg_id,
        meme_id
    ))
    .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM meme_tag WHERE meme_id = ?1", [meme_id])
        .map_err(|e| e.to_string())?;

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
    fav: bool,
    trash: bool,
) -> Result<Vec<MemeQueried>, String> {
    let guard = state.state.lock().await;
    let state = guard.as_ref().unwrap();
    let mut sql_stmt = build_search_sql(&stmt).map_err(|e| e.to_string())?;
    sql_stmt.push_str(&format!("trash == {} ", trash));
    if fav {
        sql_stmt.push_str(&format!(" AND fav == {}", fav));
    }
    sql_stmt.push_str(&format!(
        "ORDER BY update_time DESC LIMIT 30 OFFSET {}",
        30 * page
    ));

    println!("{}", sql_stmt.replace("\n", "").replace("  ", " "));
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
pub async fn delete_meme_by_id(
    state: tauri::State<'_, MemeDatabaseState>,
    id: i64,
) -> Result<(), String> {
    let mut guard = state.state.lock().await;
    let state = guard.as_mut().unwrap();
    let conn = state.conn.transaction().map_err(|e|e.to_string())?;
    conn.execute("DELETE FROM meme_tag WHERE meme_id = ?1", [id]).map_err(|e|e.to_string())?;
    conn.execute("DELETE FROM meme WHERE id = ?1", [id]).map_err(|e| e.to_string())?;
    conn.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn trash_meme_by_id(
    state: tauri::State<'_, MemeDatabaseState>,
    id: i64,
    trash: bool
) -> Result<(), String> {
    let mut guard = state.state.lock().await;
    let state = guard.as_mut().unwrap();
    let conn = state.conn.transaction().map_err(|e|e.to_string())?;
    conn.execute("UPDATE meme SET trash = ?1 WHERE id = ?2", (trash, id)).map_err(|e| e.to_string())?;
    conn.commit().map_err(|e| e.to_string())?;
    Ok(())
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

#[tauri::command]
pub async fn get_tag_keys_by_prefix(
    state: tauri::State<'_, MemeDatabaseState>,
    prefix: String,
) -> Result<Vec<String>, String> {
    let guard = state.state.lock().await;
    let state = guard.as_ref().unwrap();

    let mut query = state
        .conn
        .prepare("SELECT DISTINCT(key) FROM tag WHERE key LIKE ?1")
        .unwrap();
    let keys = query
        .query_map([format!("{}%", prefix)], |r| Ok(r.get(0).unwrap()))
        .unwrap()
        .collect::<Result<Vec<String>, Error>>()
        .map_err(|e| e.to_string())?;

    Ok(keys)
}

#[tauri::command]
pub async fn get_tags_by_prefix(
    state: tauri::State<'_, MemeDatabaseState>,
    key: String,
    prefix: String,
) -> Result<Vec<Tag>, String> {
    let guard = state.state.lock().await;
    let state = guard.as_ref().unwrap();

    let mut query = state
        .conn
        .prepare("SELECT key,value FROM tag WHERE key = ?1 AND value LIKE ?2")
        .unwrap();
    let tags = query
        .query_map([key, format!("{}%", prefix)], |r| {
            Ok(Tag {
                key: r.get("key").unwrap(),
                value: r.get("value").unwrap(),
            })
        })
        .unwrap()
        .collect::<Result<Vec<Tag>, Error>>()
        .map_err(|e| e.to_string())?;

    Ok(tags)
}

#[tauri::command]
pub async fn get_tags_fuzzy(
    state: tauri::State<'_, MemeDatabaseState>,
    keyword: String,
) -> Result<Vec<Tag>, String> {
    let guard = state.state.lock().await;
    let state = guard.as_ref().unwrap();

    let mut query = state
        .conn
        .prepare("SELECT key,value FROM tag WHERE value LIKE ?1")
        .unwrap();
    let tags = query
        .query_map([format!("%{}%", keyword)], |r| {
            Ok(Tag {
                key: r.get("key").unwrap(),
                value: r.get("value").unwrap(),
            })
        })
        .unwrap()
        .collect::<Result<Vec<Tag>, Error>>()
        .map_err(|e| e.to_string())?;
    Ok(tags)
}

fn get_relate_tag_single(key: &str, value: &str, conn: &Connection) -> Result<Vec<Tag>, String> {
    let mut query=  conn.prepare("SELECT key, value FROM meme_tag  LEFT JOIN tag ON meme_tag.tag_id = tag.id WHERE (key !=  ?1 OR value != ?2) AND meme_id IN (
	    SELECT meme_id FROM meme_tag LEFT JOIN tag ON meme_tag.tag_id = tag.id WHERE key = ?1 AND value = ?2
    )").unwrap();
    let tags = query
        .query_map([key, value], |row| {
            Ok(Tag {
                key: row.get("key").unwrap(),
                value: row.get("value").unwrap(),
            })
        })
        .unwrap()
        .collect::<Result<Vec<Tag>, Error>>()
        .map_err(|e| e.to_string())?;
    Ok(tags)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TagFreq {
    key: String,
    value: String,
    freq: u32,
}

#[tauri::command]
pub async fn get_tags_related(
    state: tauri::State<'_, MemeDatabaseState>,
    tags: Vec<Tag>,
) -> Result<Vec<TagFreq>, String> {
    let guard = state.state.lock().await;
    let state = guard.as_ref().unwrap();
    let mut freq = tags
        .iter()
        .map(|item| get_relate_tag_single(&item.key, &item.value, &state.conn))
        .collect::<Result<Vec<Vec<Tag>>, String>>()?
        .into_iter()
        .flatten()
        .filter(|x| !tags.contains(x))
        .fold(HashMap::<Tag, usize>::new(), |mut m, x| {
            *m.entry(x).or_default() += 1;
            m
        })
        .into_iter()
        .map(|v| TagFreq {
            key: v.0.key,
            value: v.0.value,
            freq: v.1 as u32,
        })
        .collect::<Vec<TagFreq>>();
    freq.sort_by(|l, r| u32::partial_cmp(&r.freq, &l.freq).unwrap());
    Ok(freq)
}
