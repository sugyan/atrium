//! Configuration for the [`BskyAgent`](super::BskyAgent).
mod file;

use crate::error::{Error, Result};
use async_trait::async_trait;
use atrium_api::agent::Session;
pub use file::FileStore;
use serde::{Deserialize, Serialize};

/// Configuration data struct for the [`BskyAgent`](super::BskyAgent).
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The base URL for the XRPC endpoint.
    pub endpoint: String,
    /// The session data.
    pub session: Option<Session>,
    /// The labelers header values.
    pub labelers_header: Option<Vec<String>>,
    /// The proxy header for service proxying.
    pub proxy_header: Option<String>,
}

impl Config {
    /// Loads the configuration from the provided loader.
    pub async fn load(loader: &impl Loader) -> Result<Self> {
        loader.load().await.map_err(Error::ConfigLoad)
    }
    /// Saves the configuration using the provided saver.
    pub async fn save(&self, saver: &impl Saver) -> Result<()> {
        saver.save(self).await.map_err(Error::ConfigSave)
    }
}

impl Default for Config {
    /// Creates a new default configuration.
    ///
    /// The default configuration uses the base URL `https://bsky.social`.
    fn default() -> Self {
        Self {
            endpoint: String::from("https://bsky.social"),
            session: None,
            labelers_header: None,
            proxy_header: None,
        }
    }
}

/// The trait for loading configuration data.
#[async_trait]
pub trait Loader {
    /// Loads the configuration data.
    async fn load(
        &self,
    ) -> core::result::Result<Config, Box<dyn std::error::Error + Send + Sync + 'static>>;
}

/// The trait for saving configuration data.
#[async_trait]
pub trait Saver {
    /// Saves the configuration data.
    async fn save(
        &self,
        config: &Config,
    ) -> core::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;
}
