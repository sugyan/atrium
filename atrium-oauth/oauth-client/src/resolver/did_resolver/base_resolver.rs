use super::{DidResolver, Error, Result};
use async_trait::async_trait;
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;
use std::sync::Arc;

pub enum Method {
    Plc,
    Web,
}

pub trait BaseResolver: Send + Sync + 'static {
    fn get_resolver(&self, method: Method) -> Arc<dyn DidResolver + Send + Sync + 'static>;
}

#[async_trait]
impl<T> DidResolver for T
where
    T: BaseResolver,
{
    async fn resolve(&self, did: &Did) -> Result<DidDocument> {
        match did.strip_prefix("did:").and_then(|s| {
            s.split_once(':').and_then(|(method, _)| match method {
                "plc" => Some(Method::Plc),
                "web" => Some(Method::Web),
                _ => None,
            })
        }) {
            Some(method) => self.get_resolver(method).resolve(did).await,
            None => Err(Error::UnsupportedMethod),
        }
    }
}
