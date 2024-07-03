use atrium_api::xrpc::error::XrpcErrorKind;
use atrium_api::xrpc::http::StatusCode;
use atrium_api::xrpc::Error as XrpcError;
use std::fmt::Debug;
use thiserror::Error;

/// Error type for this crate.
#[derive(Error, Debug)]
pub enum Error {
    #[error("xrpc response error: {0}")]
    Xrpc(Box<GenericXrpcError>),
    #[error("loading config error: {0}")]
    ConfigLoad(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("saving config error: {0}")]
    ConfigSave(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error(transparent)]
    Moderation(#[from] crate::moderation::Error),
}

#[derive(Error, Debug)]
pub enum GenericXrpcError {
    Response {
        status: StatusCode,
        error: Option<String>,
    },
    Other(String),
}

impl std::fmt::Display for GenericXrpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Response { status, error } => {
                write!(f, "{}", status.as_str())?;
                let Some(error) = &error else {
                    return Ok(());
                };
                if !error.is_empty() {
                    write!(f, " {error}")?;
                }
            }
            Self::Other(s) => {
                write!(f, "{s}")?;
            }
        }
        Ok(())
    }
}

impl<E> From<XrpcError<E>> for Error
where
    E: Debug,
{
    fn from(err: XrpcError<E>) -> Self {
        if let XrpcError::XrpcResponse(e) = err {
            Self::Xrpc(Box::new(GenericXrpcError::Response {
                status: e.status,
                error: e.error.map(|e| match e {
                    XrpcErrorKind::Custom(_) => String::from("custom error"),
                    XrpcErrorKind::Undefined(res) => res.to_string(),
                }),
            }))
        } else {
            Self::Xrpc(Box::new(GenericXrpcError::Other(format!("{err:?}"))))
        }
    }
}

/// Type alias to use this crate's [`Error`](enum@crate::Error) type in a [`Result`](core::result::Result).
pub type Result<T> = core::result::Result<T, Error>;
