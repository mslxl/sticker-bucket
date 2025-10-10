use std::{cell::Cell, sync::Mutex};

use log::{info, trace};
use rusqlite::{ffi::sqlite3_auto_extension, Connection};
use sqlite_vec::sqlite3_vec_init;

static SCHEMAS_SQL: &[&'static str] = &[
    include_str!("schemas/00-migration.sql"),
    include_str!("schemas/01-init.sql"),
    include_str!("schemas/02-asset_sha256.sql"),
];

static SQLITE_EXT_LOADED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

pub fn load_sqlite_extension() -> anyhow::Result<()> {
    let guard = SQLITE_EXT_LOADED.lock().unwrap();
    if !guard.get() {
        info!("load sqlite3 extension sqlite_vec");
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }
        guard.set(true);
    }
    Ok(())
}

pub fn run_migration(conn: &mut Connection) -> anyhow::Result<()> {
    let act = conn.transaction()?;
    act.execute_batch(SCHEMAS_SQL[0])?; // 确保 migration 表存在

    let version: usize = act.query_row_and_then(
        "SELECT IFNULL(MAX(version), 0) FROM migration;",
        [],
        |row| row.get(0),
    )?;

    // 存在未执行过的 migration 脚本
    if version < SCHEMAS_SQL.len() - 1 {
        // 执行未运行过的 SQL 文件
        for upgrade_stmt in &SCHEMAS_SQL[(version + 1)..] {
            trace!("run migration {}", upgrade_stmt);
            act.execute_batch(&upgrade_stmt)?;
        }
    }

    act.commit()?;
    Ok(())
}

mod tests {
    use rusqlite::Connection;

    use super::{load_sqlite_extension, SCHEMAS_SQL};

    #[test]
    fn test_fresh_migration() -> anyhow::Result<()> {
        load_sqlite_extension()?;
        let mut conn = Connection::open_in_memory()?;
        super::run_migration(&mut conn)?;
        Ok(())
    }

    #[test]
    fn test_continue_migration() -> anyhow::Result<()> {
        load_sqlite_extension()?;
        let mut conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMAS_SQL[0])?;
        conn.execute_batch(SCHEMAS_SQL[1])?;
        super::run_migration(&mut conn)?;
        Ok(())
    }
}
