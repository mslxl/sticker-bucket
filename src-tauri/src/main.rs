// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::PathBuf, fs};

use db::MemeDatabaseConnection;

mod db;
mod file;
mod meme;

pub struct AppDir {
    storage_dir: PathBuf,
}

fn main() {
    let storage_dir = tauri::utils::platform::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
        .join("storage");
    if !storage_dir.exists(){
        fs::create_dir_all(&storage_dir).unwrap();
    }

    tauri::Builder::default()
        .manage(AppDir {
            storage_dir: storage_dir.clone(),
        })
        .manage(MemeDatabaseConnection::open(storage_dir))
        .invoke_handler(tauri::generate_handler![
            meme::add_meme_record

        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
