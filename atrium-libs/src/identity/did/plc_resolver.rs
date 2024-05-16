use super::error::{Error, Result};
use super::{Fetch, Resolve};
use async_trait::async_trait;
use url::Url;

#[derive(Debug, Default)]
pub struct DidPlcResolver<T> {
    plc_url: String,
    timeout: Option<u64>,
    _fetcher: std::marker::PhantomData<T>,
}

impl<T> DidPlcResolver<T> {
    pub fn new(plc_url: String, timeout: Option<u64>) -> Self {
        Self {
            plc_url,
            timeout,
            _fetcher: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T> Resolve for DidPlcResolver<T>
where
    T: Fetch + Send + Sync,
{
    async fn resolve_no_check(&self, did: &str) -> Result<Option<Vec<u8>>> {
        let url = Url::parse(&format!("{}/{}", self.plc_url, urlencoding::encode(did)))?;
        T::fetch(url.as_ref(), self.timeout)
            .await
            .map_err(Error::Fetch)
    }
}
