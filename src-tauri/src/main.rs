// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod handler;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            handler::meme::open_image_and_interfere,
            handler::meme::add_meme,
            handler::meme::get_meme_by_page,
            handler::database::get_table_version,
            handler::database::get_sqlite_version,
            handler::database::get_data_dir,
            handler::database::get_image_real_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
