use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use thiserror::Error;

const SETTINGS_DIRECTORY: &str = "memelith";
const STORAGE_FILE: &str = "storage-root";
const TELEGRAM_ENABLED_FILE: &str = "telegram-enabled";
const TELEGRAM_TOKEN_FILE: &str = "telegram-token";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramSettings {
    pub enabled: bool,
    pub token: String,
}

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

pub fn load_telegram_settings() -> Result<TelegramSettings, SettingsError> {
    let directory = settings_directory()?;
    let enabled = match fs::read_to_string(directory.join(TELEGRAM_ENABLED_FILE)) {
        Ok(raw) => match raw.trim() {
            "1" | "true" => true,
            "0" | "false" => false,
            value => {
                return Err(SettingsError::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("saved Telegram enabled value `{value}` is invalid"),
                )));
            }
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
    };
    let token = match fs::read_to_string(directory.join(TELEGRAM_TOKEN_FILE)) {
        Ok(raw) => raw.trim().to_owned(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };
    Ok(TelegramSettings { enabled, token })
}

pub fn save_telegram_settings(settings: &TelegramSettings) -> Result<(), SettingsError> {
    let directory = settings_directory()?;
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join(TELEGRAM_ENABLED_FILE),
        if settings.enabled {
            "true\n"
        } else {
            "false\n"
        },
    )?;
    let token_path = directory.join(TELEGRAM_TOKEN_FILE);
    fs::write(&token_path, format!("{}\n", settings.token.trim()))?;
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&token_path)?.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(token_path, permissions)?;
    }
    Ok(())
}

fn settings_file() -> Result<PathBuf, SettingsError> {
    Ok(settings_directory()?.join(STORAGE_FILE))
}

fn settings_directory() -> Result<PathBuf, SettingsError> {
    let configuration =
        dirs::config_dir().ok_or(SettingsError::ConfigurationDirectoryUnavailable)?;
    Ok(configuration.join(SETTINGS_DIRECTORY))
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
