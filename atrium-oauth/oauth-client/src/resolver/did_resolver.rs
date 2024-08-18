mod base_resolver;
mod common_resolver;
mod plc_resolver;
mod web_resolver;

use async_trait::async_trait;
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;
use atrium_xrpc::http::uri::InvalidUri;
pub use common_resolver::{CommonResolver, CommonResolverConfig};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unsupported did method")]
    UnsupportedMethod,
    #[error(transparent)]
    Http(#[from] atrium_xrpc::http::Error),
    #[error("http client error: {0}")]
    HttpClient(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("status: {0:?}")]
    Status(Option<&'static str>),
    #[error(transparent)]
    Uri(#[from] InvalidUri),
}

pub type Result<T> = core::result::Result<T, Error>;

#[async_trait]
pub trait DidResolver: Send + Sync + 'static {
    async fn resolve(&self, did: &Did) -> Result<DidDocument>;
}
