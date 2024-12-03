use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ClientMetadata(#[from] crate::atproto::Error),
    #[error(transparent)]
    Dpop(#[from] crate::http_client::dpop::Error),
    #[error(transparent)]
    Keyset(#[from] crate::keyset::Error),
    #[error(transparent)]
    ServerAgent(#[from] crate::server_agent::Error),
    #[error(transparent)]
    Identity(#[from] atrium_identity::Error),
    #[error("authorize error: {0}")]
    Authorize(String),
    #[error("callback error: {0}")]
    Callback(String),
    #[error("state store error: {0}")]
    StateStore(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("session store error: {0}")]
    SessionStore(Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub type Result<T> = core::result::Result<T, Error>;
