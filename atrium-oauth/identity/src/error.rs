use atrium_api::types::string::Did;
use atrium_common::resolver;
use atrium_xrpc::http::uri::InvalidUri;
use atrium_xrpc::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("resource not found")]
    NotFound,
    #[error("invalid at identifier: {0}")]
    AtIdentifier(String),
    #[error("invalid did: {0}")]
    Did(String),
    #[error("invalid did document: {0}")]
    DidDocument(String),
    #[error("protected resource metadata is invalid: {0}")]
    ProtectedResourceMetadata(String),
    #[error("authorization server metadata is invalid: {0}")]
    AuthorizationServerMetadata(String),
    #[error("dns resolver error: {0}")]
    DnsResolver(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("unsupported did method: {0:?}")]
    UnsupportedDidMethod(Did),
    #[error(transparent)]
    Http(#[from] atrium_xrpc::http::Error),
    #[error("http client error: {0}")]
    HttpClient(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("http status: {0:?}")]
    HttpStatus(StatusCode),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeHtmlForm(#[from] serde_html_form::ser::Error),
    #[error(transparent)]
    Uri(#[from] InvalidUri),
}

impl From<resolver::Error> for Error {
    fn from(error: resolver::Error) -> Self {
        match error {
            resolver::Error::DnsResolver(error) => Error::DnsResolver(error),
            resolver::Error::Http(error) => Error::Http(error),
            resolver::Error::HttpClient(error) => Error::HttpClient(error),
            resolver::Error::HttpStatus(error) => Error::HttpStatus(error),
            resolver::Error::SerdeJson(error) => Error::SerdeJson(error),
            resolver::Error::SerdeHtmlForm(error) => Error::SerdeHtmlForm(error),
            resolver::Error::Uri(error) => Error::Uri(error),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
