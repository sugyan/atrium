use super::HandleResolver;
use crate::error::{Error, Result};
use crate::Resolver;
use async_trait::async_trait;
use atrium_api::types::string::{Did, Handle};
use atrium_xrpc::http::Request;
use atrium_xrpc::HttpClient;
use std::sync::Arc;

const WELL_KNWON_PATH: &str = "/.well-known/atproto-did";

#[derive(Clone, Debug)]
pub struct WellKnownHandleResolverConfig<T> {
    pub http_client: Arc<T>,
}

pub struct WellKnownHandleResolver<T> {
    http_client: Arc<T>,
}

impl<T> WellKnownHandleResolver<T> {
    pub fn new(config: WellKnownHandleResolverConfig<T>) -> Result<Self> {
        Ok(Self {
            http_client: config.http_client,
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T> Resolver for WellKnownHandleResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    type Input = Handle;
    type Output = Did;

    async fn resolve(&self, handle: &Self::Input) -> Result<Self::Output> {
        let url = format!("https://{}{WELL_KNWON_PATH}", handle.as_str());
        // TODO: no-cache?
        let res = self
            .http_client
            .send_http(Request::builder().uri(url).body(Vec::new())?)
            .await
            .map_err(Error::HttpClient)?;
        if res.status().is_success() {
            let text = String::from_utf8_lossy(res.body()).to_string();
            text.parse::<Did>().map_err(|e| Error::Did(e.to_string()))
        } else {
            Err(Error::HttpStatus(res.status()))
        }
    }
}

impl<T> HandleResolver for WellKnownHandleResolver<T> where T: HttpClient + Send + Sync + 'static {}
