use super::error::{Error, Result};
use super::{Fetcher, Resolver};
use async_trait::async_trait;

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
impl<T> Resolver for DidPlcResolver<T>
where
    T: Fetcher + Send + Sync,
{
    async fn resolve_no_check(&self, did: &str) -> Result<Option<Vec<u8>>> {
        unimplemented!()
    }
}
