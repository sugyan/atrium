use super::super::{Error, Resolver, Result};
use super::DidResolver;
use async_trait::async_trait;
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;
use atrium_xrpc::http::uri::Builder;
use atrium_xrpc::http::{Request, Uri};
use atrium_xrpc::HttpClient;
use std::sync::Arc;

const DEFAULT_PLC_DIRECTORY_URL: &str = "https://plc.directory/";

pub struct PlcDidResolverConfig<T> {
    pub plc_directory_url: Option<String>,
    pub http_client: Arc<T>,
}

pub struct PlcDidResolver<T> {
    plc_directory_url: Uri,
    http_client: Arc<T>,
}

impl<T> PlcDidResolver<T> {
    pub fn new(config: PlcDidResolverConfig<T>) -> Result<Self> {
        Ok(Self {
            plc_directory_url: config
                .plc_directory_url
                .unwrap_or(DEFAULT_PLC_DIRECTORY_URL.into())
                .parse()?,
            http_client: config.http_client,
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T> Resolver for PlcDidResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    type Input = Did;
    type Output = DidDocument;

    async fn resolve(&self, did: &Self::Input) -> Result<Self::Output> {
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
            Err(Error::HttpStatus(res.status()))
        }
    }
}

impl<T> DidResolver for PlcDidResolver<T> where T: HttpClient + Send + Sync + 'static {}
