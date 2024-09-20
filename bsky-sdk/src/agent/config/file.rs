use super::{Config, Loader, Saver};
use anyhow::anyhow;
use std::path::{Path, PathBuf};

/// An implementation of [`Loader`] and [`Saver`] that reads and writes a configuration file.
pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    /// Create a new [`FileStore`] with the given path.
    ///
    /// This `FileStore` will read and write to the file at the given path.
    /// [`Config`] data will be serialized and deserialized using the file extension.
    /// By default, this supports only `.json` files.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self { path: path.as_ref().to_path_buf() }
    }
}

impl Loader for FileStore {
    async fn load(
        &self,
    ) -> core::result::Result<Config, Box<dyn std::error::Error + Send + Sync + 'static>> {
        match self.path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => Ok(serde_json::from_str(&std::fs::read_to_string(&self.path)?)?),
            #[cfg(feature = "config-toml")]
            Some("toml") => Ok(toml::from_str(&std::fs::read_to_string(&self.path)?)?),
            _ => Err(anyhow!("Unsupported file format").into()),
        }
    }
}

impl Saver for FileStore {
    async fn save(
        &self,
        config: &Config,
    ) -> core::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        match self.path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => Ok(std::fs::write(&self.path, serde_json::to_string_pretty(config)?)?),
            #[cfg(feature = "config-toml")]
            Some("toml") => Ok(std::fs::write(&self.path, toml::to_string_pretty(config)?)?),
            _ => Err(anyhow!("Unsupported file format").into()),
        }
    }
}
