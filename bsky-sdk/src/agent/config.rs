//! Configuration for the [`BskyAgent`](super::BskyAgent).
mod file;

pub use self::file::FileStore;
use crate::error::{Error, Result};
use atrium_api::agent::atp_agent::AtpSession;
use serde::{Deserialize, Serialize};
use std::future::Future;

/// Configuration data struct for the [`BskyAgent`](super::BskyAgent).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The base URL for the XRPC endpoint.
    pub endpoint: String,
    /// The session data.
    pub session: Option<AtpSession>,
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
pub trait Loader {
    /// Loads the configuration data.
    fn load(
        &self,
    ) -> impl Future<
        Output = core::result::Result<Config, Box<dyn std::error::Error + Send + Sync + 'static>>,
    > + Send;
}

/// The trait for saving configuration data.
pub trait Saver {
    /// Saves the configuration data.
    fn save(
        &self,
        config: &Config,
    ) -> impl Future<
        Output = core::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>,
    > + Send;
}
