use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OAuthPusehedAuthorizationRequestResponse {
    pub request_uri: String,
    pub expires_in: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum OAuthTokenType {
    DPoP,
    Bearer,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: OAuthTokenType,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub expires_in: Option<i64>,
    pub sub: Option<String>,
}
