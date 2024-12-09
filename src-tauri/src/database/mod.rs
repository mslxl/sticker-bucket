use std::path::PathBuf;

use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use specta::Type;
use tokio::fs::{self};
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::database::models::{Asset, AssetImage, AssetText, AssetVideo, Collection};
use crate::database::queries::{CollectionId, GetCollectionsOrder, InsertCollectionQuery};

pub mod connection;
pub mod queries;
pub mod models;

#[derive(Type, Serialize, Deserialize, Clone, Debug)]
pub struct Prefs {
    #[serde(default = "Prefs::default_copy_assets_on_add")]
    pub copy_assets_on_add: bool,
    #[serde(default = "Prefs::default_delete_assets_on_add")]
    pub delete_assets_on_add: bool,
}

impl Prefs {
    fn default_copy_assets_on_add() -> bool {
        true
    }
    fn default_delete_assets_on_add() -> bool {
        false
    }
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            copy_assets_on_add: Self::default_copy_assets_on_add(),
            delete_assets_on_add: Self::default_delete_assets_on_add(),
        }
    }
}
pub struct Database {
    base: PathBuf,
    r2d2_pool: r2d2::Pool<SqliteConnectionManager>,
    prefs: Prefs,
}

pub type DatabaseState = Mutex<Option<Database>>;

#[tauri::command]
#[specta::specta]
pub async fn open_database(
    state: tauri::State<'_, DatabaseState>,
    path: String,
) -> Result<(), String> {
    connection::load_sqlite_extension().map_err(|e| e.to_string())?;

    let base = PathBuf::from(path);
    let manager = SqliteConnectionManager::file(base.join("memelith.sqlite"));
    let pool = r2d2::Pool::new(manager).map_err(|e| e.to_string())?;

    let mut conn = Connection::open(base.join("memelith.sqlite")).map_err(|e| e.to_string())?;
    connection::run_migration(&mut conn).map_err(|e| e.to_string())?;

    let prefs_path = base.join("prefs.json");
    let prefs = if prefs_path.exists() {
        let prefs = fs::read_to_string(prefs_path).await.unwrap_or_default();
        serde_json::from_str(&prefs).unwrap_or_default()
    } else {
        Prefs::default()
    };

    *state.lock().await = Some(Database { base, r2d2_pool: pool, prefs });
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn close_database(state: tauri::State<'_, DatabaseState>) -> Result<(), String> {
    *state.lock().await = None;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_database(state: tauri::State<'_, DatabaseState>) -> Result<String, String> {
    if let Some(db) = &*state.lock().await {
        Ok(db.base.to_str().unwrap().to_string())
    } else {
        Err("Database not open".to_string())
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_prefs(state: tauri::State<'_, DatabaseState>) -> Result<Prefs, String> {
    if let Some(db) = &mut *state.lock().await {
        Ok(db.prefs.clone())
    } else {
        Err("Database not open".to_string())
    }
}

#[tauri::command]
#[specta::specta]
pub async fn set_prefs(state: tauri::State<'_, DatabaseState>, prefs: Prefs) -> Result<(), String> {
    if let Some(db) = &mut *state.lock().await {
        db.prefs = prefs.clone();
        let prefs_path = db.base.join("prefs.json");
        fs::write(prefs_path, serde_json::to_string(&prefs).unwrap())
            .await
            .unwrap();
        Ok(())
    } else {
        Err("Database not open".to_string())
    }
}

#[derive(Type, Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum AssetEnum {
    #[serde(rename = "text")]
    Text{ c: String },
    #[serde(rename = "image")]
    Image { c: String },
    #[serde(rename = "video")]
    Video { c: String },
}

pub async fn copy_asset(path: PathBuf, base: PathBuf)->anyhow::Result<PathBuf> {
    let mut hasher = sha2::Sha256::new();
    let mut file = tokio::fs::File::open(&path).await?;
    let mut buffer = [0u8; 1024];
    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let hash = hasher.finalize();
    let hash_str = hex::encode(hash);
    let mut new_path = base.join(&hash_str[..2]).join(&hash_str[2..4]).join(&hash_str[4..]);
    new_path.set_extension(path.extension().unwrap_or_default());
    if !new_path.parent().unwrap().exists() {
        fs::create_dir_all(new_path.parent().unwrap()).await?;
    }

    fs::copy(path, &new_path).await?;
    Ok(new_path)
}
pub async fn sha256_file(path: PathBuf)->anyhow::Result<String>{
    let mut hasher = sha2::Sha256::new();
    let mut file = tokio::fs::File::open(&path).await?;
    let mut buffer = [0u8; 1024];
    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let hash = hasher.finalize();
    let hash_str = hex::encode(hash);
    Ok(hash_str)
}


#[tauri::command]
#[specta::specta]
pub async fn resolve_path(
    state: tauri::State<'_, DatabaseState>,
    path: String,
)->Result<PathBuf, String>{
    let path = PathBuf::from(path);
    if path.is_absolute(){
        Ok(path)
    }else{
        let guard = state.lock().await;
        let db = guard.as_ref().ok_or("Database not open".to_string())?;
        let new_path = db.base.join(path);
        Ok(new_path)
    }

}

#[tauri::command]
#[specta::specta]
pub async fn search_collections(
    state: tauri::State<'_, DatabaseState>,
    name: String,
) -> Result<Vec<Collection>, String> {
    let guard = state.lock().await;
    let db = guard.as_ref().ok_or("Database not open")?;
    let pool = db.r2d2_pool.clone();
    let conn = pool.get().map_err(|e| e.to_string())?;
    let collections = queries::search_collection(&conn, &name).map_err(|e| e.to_string())?;
    Ok(collections)
}

#[tauri::command]
#[specta::specta]
pub async fn get_collections(
    state: tauri::State<'_, DatabaseState>,
    order: GetCollectionsOrder,
    limit: u32,
    cursor: Option<String>,
)->Result<Vec<Collection>, String>{
    let guard = state.lock().await;
    let db = guard.as_ref().ok_or("Database not open")?;
    let pool = db.r2d2_pool.clone();
    let conn = pool.get().map_err(|e| e.to_string())?;
    let collections = queries::get_collections(&conn, order, cursor, limit).map_err(|e| e.to_string())?;

    Ok(collections)
}

#[tauri::command]
#[specta::specta]
pub async fn add_collection(state: tauri::State<'_, DatabaseState>, collection: InsertCollectionQuery)->Result<CollectionId, String>{
    let guard = state.lock().await;
    let db = guard.as_ref().ok_or("Database not open")?;
    let pool = db.r2d2_pool.clone();
    let conn = pool.get().map_err(|e| e.to_string())?;
    let collection_id = queries::insert_collection(&conn, &collection).map_err(|e| e.to_string())?;
    Ok(collection_id)
}


#[tauri::command]
#[specta::specta]
pub async fn add_sticker(
    state: tauri::State<'_, DatabaseState>,
    assets: Vec<AssetEnum>,
    name: Option<String>,
    description: Option<String>,
    collection_id: String,
    tags: Vec<String>,
    delete_original: bool,
) -> Result<(), String> {
    let guard = state.lock().await;
    let db = guard.as_ref().ok_or("Database not open")?;
    let is_copy_assets = db.prefs.copy_assets_on_add;
    let pool = db.r2d2_pool.clone();
    let conn = pool.get().map_err(|e| e.to_string())?;
    conn.execute("BEGIN TRANSACTION", []).map_err(|e| e.to_string())?;
    
    if !is_copy_assets && delete_original{
        return Err("Delete original is not allowed when copy assets is disabled".to_string());
    }


    // add assets to database
    // copy assets if needed, delete original if [[delete_original]] is true
    let mut assets_res = Vec::new();
    for asset in assets {
        let asset =  match asset {
            AssetEnum::Text{ c } => {
                let mut hasher = sha2::Sha256::new();
                hasher.update(&c.as_bytes());
                let hash = hasher.finalize();
                let hash_str = hex::encode(hash);


                queries::insert_asset(&conn, &Asset::Text(AssetText {
                    id: Uuid::new_v4().to_string(),
                    text: c,
                    create_at: chrono::Local::now().naive_local(),
                    sha256: Some(hash_str),
                })).map_err(|e| e.to_string())?
            }
            AssetEnum::Image{ c } => {

                let hash = sha256_file(PathBuf::from(&c)).await.map_err(|e|e.to_string())?;

                let image = if is_copy_assets {
                    let new_path = copy_asset(PathBuf::from(&c), db.base.clone()).await.map_err(|e|e.to_string())?;
                    if delete_original{
                        fs::remove_file(PathBuf::from(&c)).await.map_err(|e|e.to_string())?;
                    }
                    new_path.strip_prefix(&db.base)
                        .map_err(|e| e.to_string())?
                        .to_str()
                        .ok_or("Invalid path")?
                        .to_string()
                }else{
                    c
                };
                queries::insert_asset(&conn, &Asset::Image(AssetImage {
                    id: Uuid::new_v4().to_string(),
                    image,
                    create_at: chrono::Local::now().naive_local(),
                    sha256: Some(hash),
                })).map_err(|e| e.to_string())?
            }
            AssetEnum::Video{ c } => {
                let hash = sha256_file(PathBuf::from(&c)).await.map_err(|e|e.to_string())?;
                let video = if is_copy_assets {
                    let new_path = copy_asset(PathBuf::from(&c), db.base.clone()).await.map_err(|e|e.to_string())?;
                    if delete_original{
                        fs::remove_file(PathBuf::from(&c)).await.map_err(|e|e.to_string())?;
                    }
                    new_path.strip_prefix(&db.base)
                        .map_err(|e| e.to_string())?
                        .to_str()
                        .ok_or("Invalid path")?
                        .to_string()
                }else{
                    c
                };
                queries::insert_asset(&conn, &Asset::Video(AssetVideo {
                    id: Uuid::new_v4().to_string(),
                    video,
                    create_at: chrono::Local::now().naive_local(),
                    sha256: Some(hash),
                })).map_err(|e| e.to_string())?
            }
        };
        assets_res.push(asset);
    }

    // add sticker to database
    queries::insert_sticker(&conn, &queries::InsertStickerQuery {
        name,
        description,
        collection_id,
        tags,
        assets: assets_res,
    }).map_err(|e| e.to_string())?;
    
    conn.execute("COMMIT TRANSACTION", []).map_err(|e| e.to_string())?;
    Ok(())
}