// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod meme;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn get_table_version() -> u32 {
    db::DATABASE.lock().unwrap().get_table_version_code()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_table_version])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
