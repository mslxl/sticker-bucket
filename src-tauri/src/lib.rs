#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use tauri::Manager;
use tauri_plugin_decorum::WebviewWindowExt;
use tauri_specta::{collect_commands, ErrorHandlingMode};

pub mod database;
pub mod prefs;
pub mod utils;
pub mod search;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            database::open_database,
            database::close_database,
            database::get_database,
            database::get_prefs,
            database::set_prefs,
            database::resolve_path,
            database::add_sticker,
            database::search_collections,
            database::get_collections,
            database::add_collection,
            prefs::get_app_prefs::<tauri::Wry>,
            prefs::set_app_prefs::<tauri::Wry>,
            utils::whoami,
        ])
        .error_handling(ErrorHandlingMode::Throw);

    #[cfg(debug_assertions)]
    builder
        .export(
            Typescript::default().header(
                "
/* eslint-disable @typescript-eslint/ban-ts-comment */
//@ts-nocheck
/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unused-vars */
",
            ),
            "../src/lib/client.ts",
        )
        .expect("failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .clear_targets()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("logs".to_string()),
                    },
                ))
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .max_file_size(50_000 /* bytes */)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_decorum::init()) // initialize the decorum plugin
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            if let Ok(dir) = app.path().app_config_dir() {
                if !dir.exists() {
                    std::fs::create_dir_all(&dir).unwrap();
                }
            }

            app.manage(database::DatabaseState::default());
            app.manage(prefs::AppPrefsState::default());
            builder.mount_events(app);

            let main_window = app.get_webview_window("main").unwrap();
            main_window.create_overlay_titlebar().unwrap();
            #[cfg(target_os = "macos")]
            {
                main_window.set_traffic_lights_inset(12.0, 16.0).unwrap();
                main_window.make_transparent().unwrap();
                main_window.set_window_level(25).unwrap();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
