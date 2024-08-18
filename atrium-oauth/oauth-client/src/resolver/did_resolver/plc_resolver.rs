use super::{DidResolver, Error, Result};
use crate::http_client;
use async_trait::async_trait;
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;
use atrium_xrpc::http::uri::Builder;
use atrium_xrpc::http::{Request, Uri};

const DEFAULT_PLC_DIRECTORY_URL: &str = "https://plc.directory/";

pub struct PlcResolver {
    plc_directory_url: Uri,
}

impl PlcResolver {
    pub fn new(plc_directory_url: Option<String>) -> Result<Self> {
        Ok(Self {
            plc_directory_url: plc_directory_url
                .unwrap_or(DEFAULT_PLC_DIRECTORY_URL.into())
                .parse()?,
        })
    }
}

#[async_trait]
impl DidResolver for PlcResolver {
    async fn resolve(&self, did: &Did) -> Result<DidDocument> {
        let uri = Builder::from(self.plc_directory_url.clone())
            .path_and_query(format!("/{}", did.as_str()))
            .build()?;
        let client = http_client::get_http_client();
        let res = client
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
