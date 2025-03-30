use super::response::OAuthTokenType;
use atrium_api::types::string::{Datetime, Did};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TokenSet {
    pub iss: String,
    pub sub: Did,
    pub aud: String,
    pub scope: Option<String>,

    pub refresh_token: Option<String>,
    pub access_token: String,
    pub token_type: OAuthTokenType,

    pub expires_at: Option<Datetime>,
}
