#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use cmd::library::{get_default_sticky_dir, StickyDBState};
use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};

pub mod cfg;
pub mod cmd;
pub mod library;
pub mod search;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Init sticky databasestate not managed for field state on command You must call `.manage()` before using this command
            let dir = cfg::get_config_value(app.handle().clone(), "db.sticky-dir")
                .map_err(|e| e.to_string())?
                .map(|json| json.as_str().map(|s| s.to_owned()))
                .flatten()
                .or_else(|| get_default_sticky_dir(app.handle().clone()).ok())
                .expect("fail to get sticky directory");
            log::info!("Load sticky from {}", &dir);
            let state = StickyDBState::new(PathBuf::from(dir)).map_err(|e| e.to_string())?;
            app.manage(state);
            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            cmd::library::get_default_sticky_dir,
            cmd::library::create_pic_sticky,
            cmd::library::create_text_sticky,
            cmd::library::has_sticky_file,
            cmd::library::search_package,
            cmd::library::search_sticky,
            cmd::library::search_tag_ns,
            cmd::library::search_tag_value
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
