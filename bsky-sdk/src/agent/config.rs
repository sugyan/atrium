pub mod file;

use crate::error::{Error, Result};
use async_trait::async_trait;
use atrium_api::agent::Session;
pub use file::FileStore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub endpoint: String,
    pub session: Option<Session>,
    pub labelers_header: Option<Vec<String>>,
    pub proxy_header: Option<String>,
}

impl Config {
    pub async fn load(loader: &impl Loader) -> Result<Self> {
        loader.load().await.map_err(Error::ConfigLoad)
    }
    pub async fn save(&self, saver: &impl Saver) -> Result<()> {
        saver.save(self).await.map_err(Error::ConfigSave)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: String::from("https://bsky.social"),
            session: None,
            labelers_header: None,
            proxy_header: None,
        }
    }
}

#[async_trait]
pub trait Loader: Sized {
    async fn load(
        &self,
    ) -> core::result::Result<Config, Box<dyn std::error::Error + Send + Sync + 'static>>;
}

#[async_trait]
pub trait Saver {
    async fn save(
        &self,
        config: &Config,
    ) -> core::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;
}
