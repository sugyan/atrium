use super::{DidResolver, Error, Result};
use async_trait::async_trait;
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;
use atrium_xrpc::http::uri::Builder;
use atrium_xrpc::http::{Request, Uri};
use atrium_xrpc::HttpClient;
use std::sync::Arc;

const DEFAULT_PLC_DIRECTORY_URL: &str = "https://plc.directory/";

pub struct PlcResolver<T> {
    plc_directory_url: Uri,
    http_client: Arc<T>,
}

impl<T> PlcResolver<T> {
    pub fn new(plc_directory_url: Option<String>, http_client: Arc<T>) -> Result<Self> {
        Ok(Self {
            plc_directory_url: plc_directory_url
                .unwrap_or(DEFAULT_PLC_DIRECTORY_URL.into())
                .parse()?,
            http_client,
        })
    }
}

#[async_trait]
impl<T> DidResolver for PlcResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    async fn resolve(&self, did: &Did) -> Result<DidDocument> {
        let uri = Builder::from(self.plc_directory_url.clone())
            .path_and_query(format!("/{}", did.as_str()))
            .build()?;
        let res = self
            .http_client
            .send_http(Request::builder().uri(uri).body(Vec::new())?)
            .await
            .map_err(Error::HttpClient)?;
        if res.status().is_success() {
            Ok(serde_json::from_slice(res.body())?)
        } else {
            Err(Error::Status(res.status().canonical_reason()))
        }
    }
}
