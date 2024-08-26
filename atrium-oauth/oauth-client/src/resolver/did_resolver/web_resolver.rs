use super::super::{Resolver, Result};
use super::DidResolver;
use async_trait::async_trait;
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;
use atrium_xrpc::HttpClient;
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
impl<T> Resolver for WebResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    type Input = Did;
    type Output = DidDocument;

    async fn resolve(&self, _: &Self::Input) -> Result<Self::Output> {
        unimplemented!()
    }
}

impl<T> DidResolver for WebResolver<T> where T: HttpClient + Send + Sync + 'static {}
