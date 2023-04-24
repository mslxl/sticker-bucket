// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod meme;
mod handler;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            handler::database::open_image_and_interfere,
            handler::database::add_meme,
            handler::database::query_all_memes_by_page,
            handler::database::query_meme_by_id,
            handler::database::query_tag_by_meme_id,
            handler::database::query_namespace_with_prefix,
            handler::database::query_tag_value_with_prefix,
            handler::database::get_table_version,
            handler::database::get_sqlite_version,
            handler::database::get_data_dir,
            handler::database::get_image_real_path,

            handler::debug::is_debug
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
