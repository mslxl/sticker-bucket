use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{Error, Result};

/// Platform-native locations for mutable waifu-sensor runtime resources.
///
/// On Linux the roots follow the XDG base-directory specification. On macOS
/// they use `Library/Application Support` and `Library/Caches`. On Windows they
/// use the local application-data known folder.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PlatformPaths {
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub database: PathBuf,
    pub model_cache: PathBuf,
    /// Location recommended for user-installed bundle overrides.
    pub bundles: PathBuf,
}

impl PlatformPaths {
    /// Discovers platform-native data and cache roots without creating them.
    pub fn discover() -> Result<Self> {
        let data_root =
            dirs::data_local_dir().ok_or(Error::PlatformDirectoryUnavailable("local data"))?;
        let cache_root = dirs::cache_dir().ok_or(Error::PlatformDirectoryUnavailable("cache"))?;
        Ok(Self::from_roots(data_root, cache_root))
    }

    /// Builds the waifu-sensor layout below roots supplied by an embedding app.
    pub fn from_roots(data_root: impl AsRef<Path>, cache_root: impl AsRef<Path>) -> Self {
        let data_dir = data_root.as_ref().join("waifu-sensor");
        let cache_dir = cache_root.as_ref().join("waifu-sensor");
        Self {
            database: data_dir.join("waifu-sensor.sqlite3"),
            model_cache: cache_dir.join("models"),
            bundles: data_dir.join("bundles"),
            data_dir,
            cache_dir,
        }
    }

    /// Creates the mutable runtime directories represented by this layout.
    pub fn create(&self) -> Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.model_cache)?;
        std::fs::create_dir_all(&self.bundles)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn appends_a_stable_layout_to_platform_roots() {
        let paths = super::PlatformPaths::from_roots("/platform/data", "/platform/cache");

        assert_eq!(paths.data_dir, PathBuf::from("/platform/data/waifu-sensor"));
        assert_eq!(
            paths.database,
            PathBuf::from("/platform/data/waifu-sensor/waifu-sensor.sqlite3")
        );
        assert_eq!(
            paths.model_cache,
            PathBuf::from("/platform/cache/waifu-sensor/models")
        );
        assert_eq!(
            paths.bundles,
            PathBuf::from("/platform/data/waifu-sensor/bundles")
        );
    }

    #[test]
    fn creates_every_mutable_resource_directory() {
        let directory = tempfile::tempdir().unwrap();
        let paths = super::PlatformPaths::from_roots(
            directory.path().join("data"),
            directory.path().join("cache"),
        );

        paths.create().unwrap();

        assert!(paths.data_dir.is_dir());
        assert!(paths.model_cache.is_dir());
        assert!(paths.bundles.is_dir());
    }
}
