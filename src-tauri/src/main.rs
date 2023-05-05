// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};

mod db;
mod meme;
mod error;
mod search;
mod handler;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

fn build_system_tray() -> SystemTray {
    let item_quit = CustomMenuItem::new(String::from("quit"), "Quit");

    let tray_menu = SystemTrayMenu::new().add_item(item_quit);

    SystemTray::new().with_menu(tray_menu)
}
fn main() {
    let tray = build_system_tray();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            handler::database::add_meme,
            handler::database::query_meme_by_id,
            handler::database::query_tag_by_meme_id,
            handler::database::query_namespace_with_prefix,
            handler::database::query_tag_value_with_prefix,
            handler::database::query_count_memes,
            handler::database::query_count_tags,
            handler::database::get_table_version,
            handler::database::get_sqlite_version,
            handler::database::get_data_dir,
            handler::database::get_image_real_path,
            handler::database::search_memes_by_text,
            handler::local::open_image_and_interfere,
            handler::local::open_picture_list,
            handler::local::interfere_image,
            handler::debug::is_debug
        ])
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                // let item_handle = app.tray_handle().get_item(&id);
                match id.as_str() {
                    "quit" => app.exit(0),
                    _ => {}
                }
            }
            SystemTrayEvent::LeftClick { .. } => {
                let handle = app.app_handle();
                std::thread::spawn(move || {
                    let result = tauri::WindowBuilder::new(
                        &handle,
                        "main",
                        tauri::WindowUrl::App("index.html".into()),
                    )
                    .title("Meme Management")
                    .build();
                    match result {
                        Ok(_)  | Err(tauri::Error::WindowLabelAlreadyExists(_)) => {},
                        Err(e) => panic!("{:?}", e)
                    }
                });
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
