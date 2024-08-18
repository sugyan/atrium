use super::{DidResolver, Result};
use async_trait::async_trait;
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;

pub struct WebResolver;

impl WebResolver {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DidResolver for WebResolver {
    async fn resolve(&self, _: &Did) -> Result<DidDocument> {
        unimplemented!()
    }
}
