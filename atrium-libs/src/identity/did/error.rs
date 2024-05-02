use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FromUtf8(#[from] FromUtf8Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("fetch error: {0}")]
    Fetch(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Could not resolve DID: {0}")]
    DidNotFound(String),
    #[error("Poorly formatted DID: {0}")]
    PoorlyFormattedDid(String),
    #[error("Unsupported DID method: {0}")]
    UnsupportedDidMethod(String),
    #[error("Unsupported did:web paths: {0}")]
    UnsupportedDidWebPath(String),
}

pub type Result<T> = std::result::Result<T, Error>;
