mod client_metadata;
mod par_response;
mod server_metadata;

pub use client_metadata::OAuthClientMetadata;
pub use par_response::OAuthParResponse;
use serde::Serialize;
pub use server_metadata::OAuthAuthorizationServerMetadata;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenGrantType {
    AuthorizationCode,
}

// https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.3
#[derive(Serialize)]
pub struct TokenPayload {
    pub grant_type: TokenGrantType,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    // https://datatracker.ietf.org/doc/html/rfc7636#section-4.1
    pub code_verifier: Option<String>,
}
