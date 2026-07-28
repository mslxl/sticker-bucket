use std::{
    fs, io,
    path::{Path, PathBuf},
};

use thiserror::Error;

const SETTINGS_DIRECTORY: &str = "memelith";
const STORAGE_FILE: &str = "storage-root";

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("the operating system did not provide a configuration directory")]
    ConfigurationDirectoryUnavailable,

    #[error("storage path is not valid UTF-8: {0}")]
    NonUtf8StoragePath(PathBuf),

    #[error("failed to access application settings: {0}")]
    Io(#[from] io::Error),
}

pub fn load_storage_root() -> Result<Option<PathBuf>, SettingsError> {
    load_storage_root_from(&settings_file()?)
}

pub fn save_storage_root(storage_root: &Path) -> Result<(), SettingsError> {
    save_storage_root_to(&settings_file()?, storage_root)
}

fn settings_file() -> Result<PathBuf, SettingsError> {
    let configuration =
        dirs::config_dir().ok_or(SettingsError::ConfigurationDirectoryUnavailable)?;
    Ok(configuration.join(SETTINGS_DIRECTORY).join(STORAGE_FILE))
}

fn load_storage_root_from(path: &Path) -> Result<Option<PathBuf>, SettingsError> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(SettingsError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "saved storage path is empty",
        )));
    }
    Ok(Some(PathBuf::from(trimmed)))
}

fn save_storage_root_to(path: &Path, storage_root: &Path) -> Result<(), SettingsError> {
    let parent = path.parent().ok_or_else(|| {
        SettingsError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "settings file has no parent directory",
        ))
    })?;
    fs::create_dir_all(parent)?;
    let storage_root = storage_root
        .to_str()
        .ok_or_else(|| SettingsError::NonUtf8StoragePath(storage_root.to_path_buf()))?;
    fs::write(path, format!("{storage_root}\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{load_storage_root_from, save_storage_root_to};

    #[test]
    fn round_trips_the_selected_storage_root() {
        let directory = tempfile::tempdir().unwrap();
        let settings = directory.path().join("config/storage-root");
        let storage = directory.path().join("Meme Library");

        assert_eq!(load_storage_root_from(&settings).unwrap(), None);
        save_storage_root_to(&settings, &storage).unwrap();
        assert_eq!(load_storage_root_from(&settings).unwrap(), Some(storage));
    }

    #[test]
    fn rejects_an_empty_saved_storage_root() {
        let directory = tempfile::tempdir().unwrap();
        let settings = directory.path().join("storage-root");
        std::fs::write(&settings, "  \n").unwrap();

        let error = load_storage_root_from(&settings).unwrap_err();
        assert!(error.to_string().contains("saved storage path is empty"));
    }
}
