use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;
use atrium_common::resolver::Resolver;
use atrium_xrpc::HttpClient;

use super::plc_resolver::{PlcDidResolver, PlcDidResolverConfig};
use super::web_resolver::{WebDidResolver, WebDidResolverConfig};
use super::DidResolver;
use crate::error::{Error, Result};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct CommonDidResolverConfig<T> {
    pub plc_directory_url: String,
    pub http_client: Arc<T>,
}

pub struct CommonDidResolver<T> {
    plc_resolver: PlcDidResolver<T>,
    web_resolver: WebDidResolver<T>,
}

impl<T> CommonDidResolver<T> {
    pub fn new(config: CommonDidResolverConfig<T>) -> Self {
        Self {
            plc_resolver: PlcDidResolver::new(PlcDidResolverConfig {
                plc_directory_url: config.plc_directory_url,
                http_client: config.http_client.clone(),
            }),
            web_resolver: WebDidResolver::new(WebDidResolverConfig {
                http_client: config.http_client,
            }),
        }
    }
}

impl<T> Resolver for CommonDidResolver<T>
where
    PlcDidResolver<T>: DidResolver + Send + Sync + 'static,
    WebDidResolver<T>: DidResolver + Send + Sync + 'static,
{
    type Input = Did;
    type Output = DidDocument;
    type Error = Error;

    async fn resolve(&self, did: &Self::Input) -> Result<Option<Self::Output>> {
        match did.strip_prefix("did:").and_then(|s| s.split_once(':').map(|(method, _)| method)) {
            Some("plc") => self.plc_resolver.resolve(did).await,
            Some("web") => self.web_resolver.resolve(did).await,
            _ => Err(Error::UnsupportedDidMethod(did.clone())),
        }
    }
}

impl<T> DidResolver for CommonDidResolver<T> where T: HttpClient + Send + Sync + 'static {}
