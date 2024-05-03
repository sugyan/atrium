use crate::common_web::did_doc::DidDocument;
pub mod did_resolver;
mod error;
mod plc_resolver;
mod web_resolver;

use self::error::{Error, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Fetch {
    async fn fetch(
        url: &str,
        timeout: Option<u64>,
    ) -> std::result::Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>>;
}

#[async_trait]
pub trait Resolve {
    async fn resolve_no_check(&self, did: &str) -> Result<Option<Vec<u8>>>;
    async fn resolve_no_cache(&self, did: &str) -> Result<Option<DidDocument>> {
        if let Some(got) = self.resolve_no_check(did).await? {
            Ok(serde_json::from_slice(&got)?)
        } else {
            Ok(None)
        }
    }
    async fn resolve(&self, did: &str, force_refresh: bool) -> Result<Option<DidDocument>> {
        // TODO: from cache
        if let Some(got) = self.resolve_no_cache(did).await? {
            // TODO: store in cache
            Ok(Some(got))
        } else {
            // TODO: clear cache
            Ok(None)
        }
    }
    async fn ensure_resolve(&self, did: &str, force_refresh: bool) -> Result<DidDocument> {
        self.resolve(did, force_refresh)
            .await?
            .ok_or_else(|| Error::DidNotFound(did.to_string()))
    }
}
