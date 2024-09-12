use super::HandleResolver;
use crate::error::{Error, Result};
use crate::Resolver;
use async_trait::async_trait;
use atrium_api::com::atproto::identity::resolve_handle;
use atrium_api::types::string::{Did, Handle};
use atrium_xrpc::http::uri::Builder;
use atrium_xrpc::http::{Request, Uri};
use atrium_xrpc::HttpClient;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppViewHandleResolverConfig<T> {
    pub service_url: String,
    pub http_client: Arc<T>,
}

pub struct AppViewHandleResolver<T> {
    service_url: Uri,
    http_client: Arc<T>,
}

impl<T> AppViewHandleResolver<T> {
    pub fn new(config: AppViewHandleResolverConfig<T>) -> Result<Self> {
        Ok(Self {
            service_url: config.service_url.parse()?,
            http_client: config.http_client,
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T> Resolver for AppViewHandleResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    type Input = Handle;
    type Output = Did;

    async fn resolve(&self, handle: &Self::Input) -> Result<Self::Output> {
        let uri = Builder::from(self.service_url.clone())
            .path_and_query(format!(
                "/xrpc/com.atproto.identity.resolveHandle?{}",
                serde_html_form::to_string(resolve_handle::ParametersData {
                    handle: handle.clone(),
                })?
            ))
            .build()?;
        // TODO: no-cache?
        let res = self
            .http_client
            .send_http(Request::builder().uri(uri).body(Vec::new())?)
            .await
            .map_err(Error::HttpClient)?;
        if res.status().is_success() {
            Ok(serde_json::from_slice::<resolve_handle::OutputData>(res.body())?.did)
        } else {
            Err(Error::HttpStatus(res.status()))
        }
    }
}

impl<T> HandleResolver for AppViewHandleResolver<T> where T: HttpClient + Send + Sync + 'static {}
