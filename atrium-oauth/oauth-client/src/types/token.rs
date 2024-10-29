use super::response::OAuthTokenType;
use atrium_api::types::string::{Datetime, Did};
use chrono::{DateTime, Duration, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TokenSet {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub scope: Option<String>,

    pub refresh_token: Option<String>,
    pub access_token: String,
    pub token_type: OAuthTokenType,

    pub expires_at: Option<Datetime>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TokenInfo {
    pub iss: String,
    pub sub: Did,
    pub aud: String,
    pub scope: Option<String>,

    pub expires_at: Option<DateTime<FixedOffset>>,
}

impl TokenInfo {
    pub fn new(
        iss: String,
        sub: Did,
        aud: String,
        scope: Option<String>,
        expires_at: Option<DateTime<FixedOffset>>,
    ) -> Self {
        Self { iss, sub, aud, scope, expires_at }
    }

    pub fn expired(&self) -> Option<bool> {
        self.expires_at.as_ref().map(|expires_at| {
            *expires_at < (DateTime::<FixedOffset>::default() - Duration::milliseconds(5_000))
        })
    }
}
