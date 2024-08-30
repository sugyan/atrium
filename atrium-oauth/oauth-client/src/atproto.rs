use crate::keyset::Keyset;
use crate::types::{OAuthClientMetadata, TryIntoOAuthClientMetadata};
use atrium_xrpc::http::Uri;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("`client_id` must be a valid URL")]
    InvalidClientId,
    #[error("`grant_types` must include `authorization_code`")]
    InvalidGrantTypes,
    #[error("`scope` must not include `atproto`")]
    InvalidScope,
    #[error("`redirect_uris` must not be empty")]
    EmptyRedirectUris,
    #[error("`private_key_jwt` auth method requires `jwks` keys")]
    EmptyJwks,
    #[error("`private_key_jwt` auth method requires `token_endpoint_auth_signing_alg`, otherwise must not be provided")]
    AuthSigningAlg,
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrantType {
    AuthorizationCode,
    RefreshToken,
}

impl From<GrantType> for String {
    fn from(value: GrantType) -> Self {
        match value {
            GrantType::AuthorizationCode => String::from("authorization_code"),
            GrantType::RefreshToken => String::from("refresh_token"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    Atproto,
}

impl From<Scope> for String {
    fn from(value: Scope) -> Self {
        match value {
            Scope::Atproto => String::from("atproto"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtprotoLocalhostClientMetadata {
    pub redirect_uris: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtprotoClientMetadata {
    pub client_id: String,
    pub client_uri: String,
    pub redirect_uris: Vec<String>,
    pub token_endpoint_auth_method: AuthMethod,
    pub grant_types: Vec<GrantType>,
    pub scope: Vec<Scope>,
    pub jwks_uri: Option<String>,
    pub token_endpoint_auth_signing_alg: Option<String>,
}

impl TryIntoOAuthClientMetadata for AtprotoLocalhostClientMetadata {
    type Error = Error;

    fn try_into_client_metadata(self, _: &Option<Keyset>) -> Result<OAuthClientMetadata> {
        if self.redirect_uris.is_empty() {
            return Err(Error::EmptyRedirectUris);
        }
        Ok(OAuthClientMetadata {
            client_id: String::from("http://localhost"),
            client_uri: None,
            redirect_uris: self.redirect_uris,
            scope: None,       // will be set to `atproto`
            grant_types: None, // will be set to `authorization_code` and `refresh_token`
            token_endpoint_auth_method: Some(String::from("none")),
            dpop_bound_access_tokens: None, // will be set to `true`
            jwks_uri: None,
            jwks: None,
            token_endpoint_auth_signing_alg: None,
        })
    }
}

impl TryIntoOAuthClientMetadata for AtprotoClientMetadata {
    type Error = Error;

    fn try_into_client_metadata(self, keyset: &Option<Keyset>) -> Result<OAuthClientMetadata> {
        if self.client_id.parse::<Uri>().is_err() {
            return Err(Error::InvalidClientId);
        }
        if self.redirect_uris.is_empty() {
            return Err(Error::EmptyRedirectUris);
        }
        if !self.grant_types.contains(&GrantType::AuthorizationCode) {
            return Err(Error::InvalidGrantTypes);
        }
        if !self.scope.contains(&Scope::Atproto) {
            return Err(Error::InvalidScope);
        }
        let (jwks_uri, mut jwks) = (self.jwks_uri, None);
        match self.token_endpoint_auth_method {
            AuthMethod::None => {
                if self.token_endpoint_auth_signing_alg.is_some() {
                    return Err(Error::AuthSigningAlg);
                }
            }
            AuthMethod::PrivateKeyJwt => {
                if let Some(keyset) = keyset {
                    if self.token_endpoint_auth_signing_alg.is_none() {
                        return Err(Error::AuthSigningAlg);
                    }
                    if jwks_uri.is_none() {
                        jwks = Some(keyset.public_jwks());
                    }
                } else {
                    return Err(Error::EmptyJwks);
                }
            }
        }
        Ok(OAuthClientMetadata {
            client_id: self.client_id,
            client_uri: Some(self.client_uri),
            redirect_uris: self.redirect_uris,
            token_endpoint_auth_method: Some(self.token_endpoint_auth_method.into()),
            grant_types: Some(self.grant_types.into_iter().map(|v| v.into()).collect()),
            scope: Some(
                self.scope
                    .into_iter()
                    .map(|v| v.into())
                    .collect::<Vec<String>>()
                    .join(" "),
            ),
            dpop_bound_access_tokens: Some(true),
            jwks_uri,
            jwks,
            token_endpoint_auth_signing_alg: self.token_endpoint_auth_signing_alg,
        })
    }
}
