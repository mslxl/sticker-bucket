use tokio::fs;

use crate::AppDir;

#[tauri::command]
pub async fn zustand_set(state: tauri::State<'_, AppDir>, name: String, value: String) -> Result<(), String> {
  let dir = state.storage_dir.join("storage");
  if !dir.exists(){
    fs::create_dir_all(&dir).await.map_err(|e|e.to_string())?;
  }
  let mut file = dir.join(name);
  file.set_extension("json");
  fs::write(file, value).await.map_err(|e|e.to_string())?;

  Ok(())
}

#[tauri::command]
pub async fn zustand_get(state: tauri::State<'_, AppDir>, name: String) -> Result<Option<String>, String> {
  let mut file = state.storage_dir.join("storage").join(name);
  file.set_extension("json");
  if file.exists() {
    Ok(Some(fs::read_to_string(&file).await.map_err(|e|e.to_string())?))
  }else{
    Ok(None)
  }
}

#[tauri::command]
pub async fn zustand_del(state: tauri::State<'_, AppDir>, name: String) -> Result<(), String> {
  let mut file = state.storage_dir.join("storage").join(name);
  file.set_extension("json");
  if file.exists() {
    fs::remove_file(&file).await.map_err(|e|e.to_string())?;
  }
  Ok(())
}
