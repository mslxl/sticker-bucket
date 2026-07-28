use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{Error, Result};

/// Platform-native locations for mutable waifu-sensor runtime resources.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PlatformPaths {
    pub data_dir: PathBuf,
    pub database: PathBuf,
    /// Location recommended for user-installed bundle overrides.
    pub bundles: PathBuf,
}

impl PlatformPaths {
    /// Discovers the platform-native data root without creating it.
    pub fn discover() -> Result<Self> {
        let data_root =
            dirs::data_local_dir().ok_or(Error::PlatformDirectoryUnavailable("local data"))?;
        Ok(Self::from_data_root(data_root))
    }

    /// Builds the waifu-sensor layout below a root supplied by an embedding app.
    pub fn from_data_root(data_root: impl AsRef<Path>) -> Self {
        let data_dir = data_root.as_ref().join("waifu-sensor");
        Self {
            database: data_dir.join("waifu-sensor.sqlite3"),
            bundles: data_dir.join("bundles"),
            data_dir,
        }
    }

    /// Creates the mutable runtime directories represented by this layout.
    pub fn create(&self) -> Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.bundles)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn appends_a_stable_layout_to_platform_roots() {
        let paths = super::PlatformPaths::from_data_root("/platform/data");

        assert_eq!(paths.data_dir, PathBuf::from("/platform/data/waifu-sensor"));
        assert_eq!(
            paths.database,
            PathBuf::from("/platform/data/waifu-sensor/waifu-sensor.sqlite3")
        );
        assert_eq!(
            paths.bundles,
            PathBuf::from("/platform/data/waifu-sensor/bundles")
        );
    }

    #[test]
    fn creates_every_mutable_resource_directory() {
        let directory = tempfile::tempdir().unwrap();
        let paths = super::PlatformPaths::from_data_root(directory.path().join("data"));

        paths.create().unwrap();

        assert!(paths.data_dir.is_dir());
        assert!(paths.bundles.is_dir());
    }
}
