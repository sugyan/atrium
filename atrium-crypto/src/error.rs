use thiserror::Error;

/// Error types.
#[derive(Error, Debug)]
pub enum Error {
    /// Unsupported multikey type.
    #[error("Unsupported key type")]
    UnsupportedMultikeyType,
    /// Incorrect prefix for DID key.
    #[error("Incorrect prefix for did:key: {0}")]
    IncorrectDIDKeyPrefix(String),
    /// Low-S signature is not allowed.
    #[error("Low-S signature is not allowed")]
    LowSSignatureNotAllowed,
    /// Signature is invalid.
    #[error("Signature is invalid")]
    InvalidSignature,
    /// Error in [`multibase`] encoding or decoding.
    #[error(transparent)]
    Multibase(#[from] multibase::Error),
    /// Error in [`ecdsa::signature`].
    #[error(transparent)]
    Signature(#[from] ecdsa::signature::Error),
}

/// Type alias to use this library's [`Error`](crate::Error) type in a [`Result`](core::result::Result).
pub type Result<T> = std::result::Result<T, Error>;
