// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::PathBuf, fs};

use db::MemeDatabaseState;

mod db;
mod file;
mod meme;
mod zustand_storage;

pub struct AppDir {
    storage_dir: PathBuf,
}

fn main() {
    let storage_dir = tauri::utils::platform::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    if !storage_dir.exists(){
        fs::create_dir_all(&storage_dir).unwrap();
    }

    tauri::Builder::default()
        .manage(AppDir {
            storage_dir: storage_dir.clone(),
        })
        .manage(MemeDatabaseState::default())
        .invoke_handler(tauri::generate_handler![
            zustand_storage::zustand_set,
            zustand_storage::zustand_get,
            zustand_storage::zustand_del,
            meme::add_meme_record,
            meme::update_meme_record,
            meme::search_meme,
            meme::get_meme_by_id,
            meme::get_tags_by_id,
            meme::get_tag_keys_by_prefix,
            meme::get_tags_by_prefix,
            meme::get_tags_fuzzy,
            meme::get_tags_related,
            db::open_storage,
            db::get_storage,
            db::is_storage_available
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
