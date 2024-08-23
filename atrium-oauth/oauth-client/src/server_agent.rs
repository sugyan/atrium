use crate::http_client::dpop::DpopClient;
use crate::resolver::OAuthResolver;
use crate::types::{
    OAuthAuthorizationServerMetadata, OAuthClientMetadata, OAuthTokenResponse,
    PushedAuthorizationRequestParameters, TokenGrantType, TokenRequestParameters, TokenSet,
};
use atrium_api::types::string::Datetime;
use atrium_xrpc::http::{Method, Request, StatusCode};
use atrium_xrpc::HttpClient;
use chrono::TimeDelta;
use elliptic_curve::JwkEcKey;
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("no {0} endpoint available")]
    NoEndpoint(String),
    #[error("token response verification failed")]
    Token(String),
    #[error(transparent)]
    DpopClient(#[from] crate::http_client::dpop::Error),
    #[error(transparent)]
    Http(#[from] atrium_xrpc::http::Error),
    #[error("http client error: {0}")]
    HttpClient(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("http status: {0:?}")]
    HttpStatus(Option<&'static str>),
    #[error(transparent)]
    Resolver(#[from] crate::resolver::Error),
    #[error(transparent)]
    SerdeHtmlForm(#[from] serde_html_form::ser::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

#[allow(dead_code)]
pub enum OAuthRequest {
    Token(TokenRequestParameters),
    Revocation,
    Introspection,
    PushedAuthorizationRequest(PushedAuthorizationRequestParameters),
}

impl OAuthRequest {
    fn name(&self) -> String {
        String::from(match self {
            Self::Token(_) => "token",
            Self::Revocation => "revocation",
            Self::Introspection => "introspection",
            Self::PushedAuthorizationRequest(_) => "pushed_authorization_request",
        })
    }
    fn expected_status(&self) -> StatusCode {
        match self {
            Self::Token(_) => StatusCode::OK,
            Self::PushedAuthorizationRequest(_) => StatusCode::CREATED,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Serialize)]
struct RequestPayload<T>
where
    T: Serialize,
{
    client_id: String,
    #[serde(flatten)]
    parameters: T,
}

pub struct OAuthServerAgent<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    server_metadata: OAuthAuthorizationServerMetadata,
    client_metadata: OAuthClientMetadata,
    dpop_client: DpopClient<T>,
    resolver: Arc<OAuthResolver<T>>,
}

impl<T> OAuthServerAgent<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    pub fn new(
        dpop_key: JwkEcKey,
        server_metadata: OAuthAuthorizationServerMetadata,
        client_metadata: OAuthClientMetadata,
        resolver: Arc<OAuthResolver<T>>,
        http_client: Arc<T>,
    ) -> Result<Self> {
        let dpop_client = DpopClient::new(
            dpop_key,
            client_metadata.client_id.clone(),
            server_metadata
                .token_endpoint_auth_signing_alg_values_supported
                .clone(),
            http_client,
        )?;
        Ok(Self {
            server_metadata,
            client_metadata,
            dpop_client,
            resolver,
        })
    }
    /**
     * VERY IMPORTANT ! Always call this to process token responses.
     *
     * Whenever an OAuth token response is received, we **MUST** verify that the
     * "sub" is a DID, whose issuer authority is indeed the server we just
     * obtained credentials from. This check is a critical step to actually be
     * able to use the "sub" (DID) as being the actual user's identifier.
     */
    async fn verify_token_response(&self, token_response: OAuthTokenResponse) -> Result<TokenSet> {
        // ATPROTO requires that the "sub" is always present in the token response.
        let Some(sub) = &token_response.sub else {
            return Err(Error::Token("missing `sub` in token response".into()));
        };
        let (metadata, identity) = self.resolver.resolve_from_identity(sub).await?;
        if metadata.issuer != self.server_metadata.issuer {
            return Err(Error::Token("issuer mismatch".into()));
        }
        let expires_at = token_response.expires_in.and_then(|expires_in| {
            Datetime::now()
                .as_ref()
                .checked_add_signed(TimeDelta::seconds(expires_in))
                .map(Datetime::new)
        });
        Ok(TokenSet {
            sub: sub.clone(),
            aud: identity.pds,
            iss: metadata.issuer,
            scope: token_response.scope,
            id_token: token_response.id_token,
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            token_type: token_response.token_type,
            expires_at,
        })
    }
    fn build_body<S>(&self, parameters: S) -> Result<String>
    where
        S: Serialize,
    {
        Ok(serde_html_form::to_string(RequestPayload {
            client_id: self.client_metadata.client_id.clone(),
            parameters,
        })?)
    }
    fn endpoint(&self, request: &OAuthRequest) -> Option<&String> {
        match request {
            OAuthRequest::Token(_) => Some(&self.server_metadata.token_endpoint),
            OAuthRequest::Revocation => self.server_metadata.revocation_endpoint.as_ref(),
            OAuthRequest::Introspection => self.server_metadata.introspection_endpoint.as_ref(),
            OAuthRequest::PushedAuthorizationRequest(_) => self
                .server_metadata
                .pushed_authorization_request_endpoint
                .as_ref(),
        }
    }
    pub async fn exchange_code(&self, code: &str, verifier: &str) -> Result<TokenSet> {
        self.verify_token_response(
            self.request(OAuthRequest::Token(TokenRequestParameters {
                grant_type: TokenGrantType::AuthorizationCode,
                code: code.into(),
                redirect_uri: self.client_metadata.redirect_uris[0].clone(), // ?
                code_verifier: verifier.into(),
            }))
            .await?,
        )
        .await
    }
    pub async fn request<O>(&self, request: OAuthRequest) -> Result<O>
    where
        O: serde::de::DeserializeOwned,
    {
        let Some(url) = self.endpoint(&request) else {
            return Err(Error::NoEndpoint(request.name()));
        };
        let body = match &request {
            OAuthRequest::Token(params) => self.build_body(params)?,
            OAuthRequest::PushedAuthorizationRequest(params) => self.build_body(params)?,
            _ => unimplemented!(),
        };
        println!("body: {body}");
        let req = Request::builder()
            .uri(url)
            .method(Method::POST)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.into_bytes())?;
        let res = self
            .dpop_client
            .send_http(req)
            .await
            .map_err(Error::HttpClient)?;
        if res.status() == request.expected_status() {
            Ok(serde_json::from_slice(res.body())?)
        } else {
            println!("{}: {}", res.status(), String::from_utf8_lossy(res.body()));
            Err(Error::HttpStatus(res.status().canonical_reason()))
        }
    }
    // fn build_auth(&self, endpoint: OAuthEndpointName) -> Result<String> {
    //     if let Some(methods_supported) = self.methods_supported(endpoint) {
    //         if let Some(method) = self.client_metadata.token_endpoint_auth_method.as_deref() {
    //             if !methods_supported.contains(&method.into()) {
    //                 return Err(Error::UnsupportedAuthMethod(endpoint));
    //             }
    //         }
    //     }
    //     match self.client_metadata.token_endpoint_auth_method.as_deref() {
    //         Some("none") => Ok(self.client_metadata.client_id.clone()),
    //         Some("private_key_jwt") => {
    //             // TODO
    //             todo!()
    //         }
    //         _ => Err(Error::UnsupportedAuthMethod(endpoint)),
    //     }
    // }
    // fn methods_supported(&self, endpoint: OAuthEndpointName) -> Option<&Vec<String>> {
    //     match endpoint {
    //         OAuthEndpointName::Token | OAuthEndpointName::PushedAuthorizationRequest => self
    //             .server_metadata
    //             .token_endpoint_auth_methods_supported
    //             .as_ref(),
    //         OAuthEndpointName::Revocation => self
    //             .server_metadata
    //             .revocation_endpoint_auth_methods_supported
    //             .as_ref()
    //             .or(self
    //                 .server_metadata
    //                 .token_endpoint_auth_methods_supported
    //                 .as_ref()),
    //         OAuthEndpointName::Introspection => self
    //             .server_metadata
    //             .introspection_endpoint_auth_methods_supported
    //             .as_ref()
    //             .or(self
    //                 .server_metadata
    //                 .token_endpoint_auth_methods_supported
    //                 .as_ref()),
    //     }
    // }
}
