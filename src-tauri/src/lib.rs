use specta_typescript::Typescript;
use tauri::{generate_handler, Manager};
use tauri_specta::{collect_commands, collect_events, Builder};

pub mod error;
pub mod prefs;

mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let prefs_builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            prefs::set_global_prefs::<tauri::Wry>,
            prefs::get_global_prefs
        ])
        .events(collect_events![prefs::GlobalPrefsChangedEvent]);
    #[cfg(debug_assertions)] // <- Only export on non-release builds
    prefs_builder
        .export(
            Typescript::default().header("//@ts-nocheck\n"),
            "../src/prefs/type.ts",
        )
        .expect("Failed to export prefs typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            app.manage(prefs::GlobalPrefsState::init(app.path())?);
            prefs_builder.mount_events(app);
            Ok(())
        })
        .invoke_handler(generate_handler![
            // prefs builder
            prefs::set_global_prefs,
            prefs::get_global_prefs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
