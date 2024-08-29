use specta_typescript::Typescript;
use tauri::{generate_handler, Manager};
use tauri_specta::{collect_commands, collect_events, Builder};
use tauri_plugin_decorum::WebviewWindowExt;
pub mod error;
pub mod prefs;

mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let prefs_builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            prefs::set_global_prefs::<tauri::Wry>,
            prefs::get_global_prefs,
            prefs::is_debug_mode
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_decorum::init())
        .setup(move |app| {
            let main_window = app.get_webview_window("main").unwrap();
            main_window.create_overlay_titlebar().unwrap();

            // Some macOS-specific helpers
			#[cfg(target_os = "macos")] {
				// Set a custom inset to the traffic lights
				main_window.set_traffic_lights_inset(12.0, 16.0).unwrap();

				// Make window transparent without privateApi
				main_window.make_transparent().unwrap();

				// Set window level
				// NSWindowLevel: https://developer.apple.com/documentation/appkit/nswindowlevel
				main_window.set_window_level(25).unwrap()
			}

            app.manage(prefs::GlobalPrefsState::init(app.path())?);
            prefs_builder.mount_events(app);
            Ok(())
        })
        .invoke_handler(generate_handler![
            // prefs builder
            prefs::set_global_prefs,
            prefs::get_global_prefs,
            prefs::is_debug_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
