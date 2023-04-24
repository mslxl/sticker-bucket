#[tauri::command]
pub fn is_debug() -> bool {
  #[cfg(debug_assertions)]
  return true;
  #[cfg(not(debug_assertions))]
  return false;
}