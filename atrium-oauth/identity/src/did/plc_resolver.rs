use super::DidResolver;
use crate::error::{Error, Result};
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;
use atrium_common::resolver::Resolver;
use atrium_xrpc::http::uri::Builder;
use atrium_xrpc::http::{Request, Uri};
use atrium_xrpc::HttpClient;
use std::sync::Arc;

pub const DEFAULT_PLC_DIRECTORY_URL: &str = "https://plc.directory/";

#[derive(Clone, Debug)]
pub struct PlcDidResolverConfig<T> {
    pub plc_directory_url: String,
    pub http_client: Arc<T>,
}

pub struct PlcDidResolver<T> {
    plc_directory_url: String,
    http_client: Arc<T>,
}

impl<T> PlcDidResolver<T> {
    pub fn new(config: PlcDidResolverConfig<T>) -> Self {
        Self { plc_directory_url: config.plc_directory_url, http_client: config.http_client }
    }
}

impl<T> Resolver for PlcDidResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    type Input = Did;
    type Output = DidDocument;
    type Error = Error;

    async fn resolve(&self, did: &Self::Input) -> Result<Option<Self::Output>> {
        let uri = Builder::from(self.plc_directory_url.parse::<Uri>()?)
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
