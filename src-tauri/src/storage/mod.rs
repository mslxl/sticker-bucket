pub mod db;
pub mod prefs;
use anyhow::Result;
use std::path::PathBuf;

use prefs::StoragePrefs;
use tokio::{fs, sync::RwLock};

use crate::error::Error;

pub struct Storage {
    location: PathBuf,
    prefs: StoragePrefs,
}

impl Storage {
    async fn lock(&mut self) -> Result<()> {
        let lockfile = self.location.join(".lock");
        if lockfile.exists() {
            return Err(Error::LibraryLocked.into());
        }
        fs::write(
            &lockfile,
            "DO NOT DELETE THIS FILE IF YOU DO NOT KNOW WHAT YOU ARE DO!",
        )
        .await?;
        Ok(())
    }
    async fn unlock(&mut self) -> Result<()> {
        let lockfile = self.location.join(".lock");
        if lockfile.exists() {
            fs::remove_file(&lockfile).await?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct OpenedStorageState {
    storage: RwLock<Option<Storage>>,
}

#[tauri::command]
#[specta::specta]
async fn open_storage(
    state: tauri::State<'_, OpenedStorageState>,
    path: PathBuf,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn get_current_storage(state: tauri::State<'_, OpenedStorageState>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn close_storage(state: tauri::State<'_, OpenedStorageState>) -> Result<(), String> {
    Ok(())
}
