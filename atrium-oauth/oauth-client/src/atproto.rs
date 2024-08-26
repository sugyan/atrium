use crate::types::OAuthClientMetadata;
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
    OfflineAccess, // will be removed: https://github.com/bluesky-social/atproto/pull/2731/files#diff-8655bc89b9f05348fdc55d73ed4298ec3ff0edd03fcb601b30c514e61465ede7L207-L211
}

impl From<Scope> for String {
    fn from(value: Scope) -> Self {
        match value {
            Scope::Atproto => String::from("atproto"),
            Scope::OfflineAccess => String::from("offline_access"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtprotoLocalhostClientMetadata {
    pub redirect_uris: Vec<String>,
}

impl TryFrom<AtprotoLocalhostClientMetadata> for OAuthClientMetadata {
    type Error = Error;

    fn try_from(value: AtprotoLocalhostClientMetadata) -> Result<Self> {
        if value.redirect_uris.is_empty() {
            return Err(Error::EmptyRedirectUris);
        }
        Ok(OAuthClientMetadata {
            client_id: String::from("http://localhost"),
            client_uri: None,
            redirect_uris: value.redirect_uris,
            scope: Some(String::from("atproto")),
            grant_types: None, // will be set to `authorization_code` and `refresh_token`
            token_endpoint_auth_method: None, // will be set to `none`
            dpop_bound_access_tokens: None, // will be set to `true`
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtprotoClientMetadata {
    pub client_id: String,
    pub client_uri: String,
    pub redirect_uris: Vec<String>,
    pub token_endpoint_auth_method: AuthMethod,
    pub grant_types: Vec<GrantType>,
    pub scope: Vec<Scope>,
}

impl TryFrom<AtprotoClientMetadata> for OAuthClientMetadata {
    type Error = Error;

    fn try_from(value: AtprotoClientMetadata) -> Result<Self> {
        if value.client_id.parse::<Uri>().is_err() {
            return Err(Error::InvalidClientId);
        }
        if value.redirect_uris.is_empty() {
            return Err(Error::EmptyRedirectUris);
        }
        if !value.grant_types.contains(&GrantType::AuthorizationCode) {
            return Err(Error::InvalidGrantTypes);
        }
        if !value.scope.contains(&Scope::Atproto) {
            return Err(Error::InvalidScope);
        }

        // TODO: jwks

        Ok(OAuthClientMetadata {
            client_id: value.client_id,
            client_uri: Some(value.client_uri),
            redirect_uris: value.redirect_uris,
            token_endpoint_auth_method: Some(value.token_endpoint_auth_method.into()),
            grant_types: Some(value.grant_types.into_iter().map(|v| v.into()).collect()),
            scope: Some(
                value
                    .scope
                    .into_iter()
                    .map(|v| v.into())
                    .collect::<Vec<String>>()
                    .join(" "),
            ),
            dpop_bound_access_tokens: Some(true),
        })
    }
}
