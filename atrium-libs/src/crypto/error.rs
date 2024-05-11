use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unsupported key type")]
    UnsupportedMultikeyType,
    #[error("Incorrect prefix for did:key: {0}")]
    IncorrectDIDKeyPrefix(String),
    #[error(transparent)]
    Multibase(#[from] multibase::Error),
    #[error(transparent)]
    Signature(#[from] ecdsa::signature::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
