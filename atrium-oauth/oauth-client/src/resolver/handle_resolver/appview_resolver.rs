use super::super::{Error, Resolver, Result};
use super::HandleResolver;
use async_trait::async_trait;
use atrium_api::com::atproto::identity::resolve_handle;
use atrium_api::types::string::{Did, Handle};
use atrium_xrpc::http::uri::Builder;
use atrium_xrpc::http::{Request, Uri};
use atrium_xrpc::HttpClient;
use std::sync::Arc;

#[derive(Debug)]
pub struct AppViewResolver<T> {
    service: Uri,
    http_client: Arc<T>,
}

impl<T> AppViewResolver<T> {
    pub fn new(service: Uri, http_client: Arc<T>) -> Self {
        Self {
            service,
            http_client,
        }
    }
}

#[async_trait]
impl<T> Resolver for AppViewResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    type Input = Handle;
    type Output = Did;

    async fn resolve(&self, handle: &Self::Input) -> Result<Self::Output> {
        let uri = Builder::from(self.service.clone())
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
            Err(Error::HttpStatus(res.status().canonical_reason()))
        }
    }
}

impl<T> HandleResolver for AppViewResolver<T> where T: HttpClient + Send + Sync + 'static {}
