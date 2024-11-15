use atrium_xrpc::http::uri::InvalidUri;
use atrium_xrpc::http::StatusCode;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("resource not found")]
    NotFound,
    #[error("dns resolver error: {0}")]
    DnsResolver(Box<dyn std::error::Error + Send + Sync + 'static>),
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
