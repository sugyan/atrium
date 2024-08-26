use super::super::{Error, Resolver, Result};
use super::DidResolver;
use async_trait::async_trait;
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;
use std::sync::Arc;

pub enum Method {
    Plc,
    Web,
}

pub trait BaseResolver {
    fn get_resolver(&self, method: Method) -> Arc<dyn DidResolver + Send + Sync + 'static>;
}

#[async_trait]
impl<T> Resolver for T
where
    T: BaseResolver + Send + Sync + 'static,
{
    type Input = Did;
    type Output = DidDocument;

    async fn resolve(&self, did: &Did) -> Result<DidDocument> {
        match did.strip_prefix("did:").and_then(|s| {
            s.split_once(':').and_then(|(method, _)| match method {
                "plc" => Some(Method::Plc),
                "web" => Some(Method::Web),
                _ => None,
            })
        }) {
            Some(method) => self.get_resolver(method).resolve(did).await,
            None => Err(Error::UnsupportedDidMethod(did.clone())),
        }
    }
}

impl<T> DidResolver for T where T: BaseResolver + Send + Sync + 'static {}
