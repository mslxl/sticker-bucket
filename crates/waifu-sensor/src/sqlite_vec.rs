use std::{ffi::c_char, ptr};

use rusqlite::{Connection, ffi};

use crate::{Error, Result};

// sqlite-vec's Rust crate exposes the symbol with a zero-argument declaration for
// sqlite3_auto_extension. We need the real extension signature because callers
// hand waifu-sensor an already-open Connection.
unsafe extern "C" {
    #[link_name = "sqlite3_vec_init"]
    fn sqlite3_vec_init_for_connection(
        database: *mut ffi::sqlite3,
        error_message: *mut *mut c_char,
        api: *const ffi::sqlite3_api_routines,
    ) -> i32;
}

pub(crate) fn initialize(connection: &Connection) -> Result<()> {
    // SAFETY: Connection::handle returns this live connection's sqlite3 pointer.
    // sqlite-vec is statically compiled with SQLITE_CORE, so its initializer accepts
    // a null API table. No error string is requested, and the call completes before
    // the borrowed Connection can be used or dropped elsewhere.
    let status = unsafe {
        sqlite3_vec_init_for_connection(connection.handle(), ptr::null_mut(), ptr::null())
    };
    if status != ffi::SQLITE_OK {
        return Err(Error::SqliteVecInitialization(status));
    }

    let version: String = connection.query_row("SELECT vec_version()", [], |row| row.get(0))?;
    if version.trim().is_empty() {
        return Err(Error::InvalidBundle(
            "sqlite-vec returned an empty version".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    #[test]
    fn initializes_sqlite_vec_on_an_already_open_connection() {
        let connection = Connection::open_in_memory().unwrap();

        super::initialize(&connection).unwrap();
        connection
            .execute(
                "CREATE VIRTUAL TABLE vectors USING vec0(embedding float[2])",
                [],
            )
            .unwrap();

        let version: String = connection
            .query_row("SELECT vec_version()", [], |row| row.get(0))
            .unwrap();
        assert!(version.starts_with("v0."), "unexpected version: {version}");
    }
}
