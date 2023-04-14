use crate::db::DATABASE;


#[tauri::command]
pub fn get_table_version() -> u32 {
    DATABASE.lock().unwrap().get_table_version_code()
}