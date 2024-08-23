use super::{DidResolver, Result};
use async_trait::async_trait;
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;
use std::sync::Arc;

pub struct WebResolver<T> {
    #[allow(dead_code)]
    http_client: Arc<T>,
}

impl<T> WebResolver<T> {
    pub fn new(http_client: Arc<T>) -> Self {
        Self { http_client }
    }
}

#[async_trait]
impl<T> DidResolver for WebResolver<T>
where
    T: Send + Sync + 'static,
{
    async fn resolve(&self, _: &Did) -> Result<DidDocument> {
        unimplemented!()
    }
}
