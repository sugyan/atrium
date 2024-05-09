use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unsupported key type")]
    UnsupportedMultikeyType,
    #[error(transparent)]
    Multibase(#[from] multibase::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
