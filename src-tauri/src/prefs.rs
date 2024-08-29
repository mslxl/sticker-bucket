use std::cell::Cell;
use std::fs::{create_dir_all, read_to_string};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Typescript;
use tauri::path::PathResolver;
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{App, Emitter, Manager, Runtime};
use tauri_specta::{collect_commands, collect_events, Event};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppGlobalCfg {
    #[serde(default = "AppGlobalCfg::default_lng")]
    pub lng: String,
    #[serde(default = "AppGlobalCfg::default_history")]
    pub history: Vec<String>,
    #[serde(default = "AppGlobalCfg::default_last_open")]
    pub last_open: Option<String>,
    #[serde(default = "AppGlobalCfg::default_always_open_last")]
    pub always_open_last: bool,
    #[serde(default = "AppGlobalCfg::default_dark_mode")]
    pub dark_mode: bool,
    #[serde(default = "AppGlobalCfg::default_accent_color")]
    pub accent_color: String,
}

impl AppGlobalCfg {
    fn default_lng() -> String{
        String::from("en")
    }
    fn default_history() -> Vec<(String)> {
        Vec::new()
    }
    fn default_last_open() -> Option<String> {
        None
    }
    fn default_always_open_last() -> bool {
        false
    }
    fn default_dark_mode() -> bool {
        false
    }
    fn default_accent_color() -> String {
        String::from("blue")
    }
}

impl Default for AppGlobalCfg {
    fn default() -> Self {
        Self {
            lng: Self::default_lng(),
            history: Self::default_history(),
            last_open: Self::default_last_open(),
            always_open_last: Self::default_always_open_last(),
            dark_mode: Self::default_dark_mode(),
            accent_color: Self::default_accent_color(),
        }
    }
}

impl AppGlobalCfg {
    fn load<R: Runtime>(resolver: &PathResolver<R>) -> Result<Self> {
        let config_dir = resolver.app_config_dir()?;
        if !config_dir.exists() {
            create_dir_all(&config_dir)?;
        }
        let config_file = config_dir.join("config.toml");
        let cfg = if config_file.exists() {
            toml::from_str(&read_to_string(&config_file)?)?
        } else {
            AppGlobalCfg::default()
        };
       
        Ok(cfg)
    }
    async fn save<R:Runtime>(&self, resolver: &PathResolver<R>)->Result<()>{
        let config_dir = resolver.app_config_dir()?;
        if !config_dir.exists() {
            tokio::fs::create_dir_all(&config_dir).await?;
        }
        let config_file = config_dir.join("config.toml");
        tokio::fs::write(config_file, toml::to_string_pretty(self)?).await?;
        Ok(())
    }
}

pub struct GlobalPrefsState {
    content: RwLock<AppGlobalCfg>,
}
impl GlobalPrefsState {
    pub fn init<R: Runtime>(resolver: &PathResolver<R>) -> Result<Self> {
        Ok(Self {
            content: RwLock::new(AppGlobalCfg::load(resolver)?),
        })
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_global_prefs(
    state: tauri::State<'_, GlobalPrefsState>,
) -> Result<AppGlobalCfg, String> {
    let reader = state.content.read().await;
    Ok(reader.clone())
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct GlobalPrefsChangedEvent {
    tag: String,
}

#[tauri::command]
#[specta::specta]
pub async fn set_global_prefs<R: Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, GlobalPrefsState>,
    prefs: AppGlobalCfg,
    tag: String,
) -> Result<(), String> {
    let mut reader = state.content.write().await;
    prefs.save(app.path()).await.map_err(|e| e.to_string())?;
    *reader = prefs;
    GlobalPrefsChangedEvent { tag }
        .emit(&app)
        .map_err(|e| e.to_string())?;

    Ok(())
}


#[tauri::command]
#[specta::specta]
pub async fn is_debug_mode() -> Result<bool, String> {
    #[cfg(debug_assertions)]
    return Ok(true);

    Ok(false)
}