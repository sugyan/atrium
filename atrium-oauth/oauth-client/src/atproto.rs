use crate::types::OAuthClientMetadata;
use atrium_xrpc::http::Uri;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("client_id must be a valid URL")]
    InvalidClientId,
    #[error("redirect_uris must not be empty")]
    EmptyRedirectUris,
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    None,
    // https://openid.net/specs/openid-connect-core-1_0.html#ClientAuthentication
    PrivateKeyJwt,
}

impl From<AuthMethod> for String {
    fn from(value: AuthMethod) -> Self {
        match value {
            AuthMethod::None => String::from("none"),
            AuthMethod::PrivateKeyJwt => String::from("private_key_jwt"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ClientMetadata {
    pub client_id: String,
    pub redirect_uris: Vec<String>,
    pub token_endpoint_auth_method: AuthMethod,
}

impl ClientMetadata {
    pub fn validate(self) -> Result<OAuthClientMetadata> {
        if self.redirect_uris.is_empty() {
            return Err(Error::EmptyRedirectUris);
        }

        // TODO: jwks

        if self.client_id.parse::<Uri>().is_err() {
            return Err(Error::InvalidClientId);
        }
        Ok(OAuthClientMetadata {
            client_id: self.client_id,
            redirect_uris: self.redirect_uris,
            token_endpoint_auth_method: Some(self.token_endpoint_auth_method.into()),
        })
    }
}
