use std::path::PathBuf;

use anyhow::Result;
use serde_json::Value;
use tauri::{Manager, Runtime};
use tauri_plugin_store::{with_store, Store, StoreCollection};

fn with_config_store<F, R, T>(app: tauri::AppHandle<R>, f: F) -> Result<T>
where
    F: FnOnce(&mut Store<R>) -> tauri_plugin_store::Result<T>,
    R: Runtime,
{
    let state_app_handle = app.clone();
    let stores = state_app_handle.state::<StoreCollection<R>>();
    let path = PathBuf::from("stickerbucket.json");
    let value = with_store(app, stores, path, f)?;
    Ok(value)
}

pub fn get_config_value<R: Runtime>(app: tauri::AppHandle<R>, key: &str) -> Result<Option<Value>> {
    with_config_store(app, |store| Ok(store.get(key).cloned()))
}

pub fn del_config_value<R: Runtime>(app: tauri::AppHandle<R>, key: &str) -> Result<bool> {
    with_config_store(app, |store| store.delete(key))
}

pub fn set_config_value<R: Runtime>(
    app: tauri::AppHandle<R>,
    key: String,
    value: Value,
) -> Result<()> {
    with_config_store(app, |store| store.insert(key, value))
}
