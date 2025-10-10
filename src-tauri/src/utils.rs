
#[tauri::command]
#[specta::specta]
pub async fn whoami()->Result<String, String> {
    Ok(whoami::username())
}