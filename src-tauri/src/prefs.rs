use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{Manager, Runtime};
use tokio::{fs, sync::RwLock};

#[derive(Type, Serialize, Deserialize, Clone, Debug)]
pub struct AppPrefs {
    database_history: Vec<String>,
}

impl Default for AppPrefs {
    fn default() -> Self {
        Self {
            database_history: vec![],
        }
    }
}

pub struct AppPrefsState {
    app_prefs: RwLock<Option<AppPrefs>>,
}

impl Default for AppPrefsState {
    fn default() -> Self {
        Self {
            app_prefs: RwLock::new(None),
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_app_prefs<R: Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, AppPrefsState>,
) -> Result<AppPrefs, String> {
    {
        let app_prefs = state.app_prefs.read().await;
        if let Some(prefs) = app_prefs.as_ref() {
            return Ok(prefs.clone());
        }
    }

    let app_prefs_path = app.path().app_config_dir().unwrap().join("app_prefs.json");
    log::trace!("app_prefs_path: {:?}", app_prefs_path);

    let app_prefs = if app_prefs_path.exists() {
        let app_prefs = fs::read_to_string(app_prefs_path).await.unwrap_or_default();
        let app_prefs: AppPrefs = serde_json::from_str(&app_prefs).unwrap_or_default();
        log::trace!("use app_prefs: {:?}", &app_prefs);
        app_prefs
    } else {
        let app_prefs = AppPrefs::default();
        log::trace!("use default app_prefs: {:?}", &app_prefs);
        app_prefs
    };

    {
        *state.app_prefs.write().await = Some(app_prefs.clone());
    }

    Ok(app_prefs)
}

#[tauri::command]
#[specta::specta]
pub async fn set_app_prefs<R: Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, AppPrefsState>,
    app_prefs: AppPrefs,
) -> Result<(), String> {
    *state.app_prefs.write().await = Some(app_prefs.clone());
    let app_prefs_path = app.path().app_config_dir().unwrap().join("app_prefs.json");
    fs::write(app_prefs_path, serde_json::to_string(&app_prefs).unwrap())
        .await
        .unwrap();
    Ok(())
}
