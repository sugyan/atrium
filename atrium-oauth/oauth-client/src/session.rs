use atrium_api::types::string::Did;
use jose_jwk::Key;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::TokenSet;

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenError {
    #[error("The session for {0:?} could not be refreshed")]
    Refresh(Did),
    #[error("The session for {0:?} was revoked")]
    Revoked(Did),
    #[error("The session for {0:?} is invalid")]
    Invalid(Did),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub dpop_key: Key,
    pub token_set: TokenSet,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Updated { inner: Session, sub: Did },
    Deleted { inner: Session, cause: TokenError },
}
