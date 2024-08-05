#![doc = "Error types."]
use thiserror::Error;

/// Error type for this crate.
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IpldCoreSerde(#[from] ipld_core::serde::SerdeError),
    #[error("not allowed in ATProtocol")]
    NotAllowed,
}

/// Type alias to use this library's [`Error`] type in a [`Result`](core::result::Result).
pub type Result<T> = core::result::Result<T, Error>;
