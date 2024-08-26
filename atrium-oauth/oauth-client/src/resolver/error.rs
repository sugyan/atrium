use atrium_api::types::string::Did;
use atrium_xrpc::http::uri::InvalidUri;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid at identifier: {0}")]
    AtIdentifier(&'static str),
    #[error("invalid did document: {0}")]
    DidDocument(String),
    #[error("protected resource metadata is invalid: {0}")]
    ProtectedResourceMetadata(String),
    #[error("authorization server metadata is invalid: {0}")]
    AuthorizationServerMetadata(String),
    #[error("unsupported did method: {0:?}")]
    UnsupportedDidMethod(Did),
    #[error(transparent)]
    Http(#[from] atrium_xrpc::http::Error),
    #[error("http client error: {0}")]
    HttpClient(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("http status: {0:?}")]
    HttpStatus(Option<&'static str>),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeHtmlForm(#[from] serde_html_form::ser::Error),
    #[error(transparent)]
    Uri(#[from] InvalidUri),
}

pub type Result<T> = core::result::Result<T, Error>;
