use super::{Error, HandleResolver, Result};
use crate::http_client;
use async_trait::async_trait;
use atrium_api::com::atproto::identity::resolve_handle;
use atrium_api::types::string::{Did, Handle};
use atrium_xrpc::http::uri::Builder;
use atrium_xrpc::http::{Request, Uri};

#[derive(Debug)]
pub struct AppViewResolver {
    service: Uri,
}

impl AppViewResolver {
    pub fn new(service: Uri) -> Self {
        Self { service }
    }
}

#[async_trait]
impl HandleResolver for AppViewResolver {
    async fn resolve(&self, handle: &Handle) -> Result<Did> {
        let uri = Builder::from(self.service.clone())
            .path_and_query(format!(
                "/xrpc/com.atproto.identity.resolveHandle?{}",
                serde_html_form::to_string(resolve_handle::ParametersData {
                    handle: handle.clone(),
                })?
            ))
            .build()?;
        // TODO: no-cache?
        let client = http_client::get_http_client();
        let res = client
            .send_http(Request::builder().uri(uri).body(Vec::new())?)
            .await
            .map_err(Error::HttpClient)?;
        if res.status().is_success() {
            Ok(serde_json::from_slice::<resolve_handle::OutputData>(res.body())?.did)
        } else {
            Err(Error::Status(res.status().canonical_reason()))
        }
    }
}
