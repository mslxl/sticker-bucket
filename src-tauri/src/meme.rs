use std::path::PathBuf;

use rusqlite::{Connection, OptionalExtension};

use crate::{db::MemeDatabaseConnection, file::copy_to_storage, AppDir};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Tag {
    key: String,
    value: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MemeToAdd {
    name: String,
    description: String,
    ty: String,
    content: Option<String>,
    fav: bool,
    tags: Vec<Tag>,
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
    db_state: tauri::State<'_, MemeDatabaseConnection>,
    base_state: tauri::State<'_, AppDir>,
    mut item: MemeToAdd,
) -> Result<(), String> {
    let mut connection = db_state.conn.lock().await;
    let conn = connection.transaction().map_err(|e| e.to_string())?;
    let tag_id = item
        .tags
        .iter()
        .map(|t| make_tag(&conn, &t.key, &t.value))
        .collect::<Result<Vec<i64>, String>>()?;

    if item.ty == "image" {
        let new_content = copy_to_storage(
            &base_state.storage_dir,
            PathBuf::from(&item.content.unwrap_or(String::from(""))),
        )
        .map_err(|e| e.to_string())?;
        item.content = Some(new_content)
    }

    conn.execute(
        "INSERT INTO meme(name, description, ty, content, fav, pkg_id)
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
