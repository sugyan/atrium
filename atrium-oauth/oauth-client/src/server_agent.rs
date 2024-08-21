use crate::http_client::dpop::DpopClient;
use crate::types::{
    OAuthAuthorizationServerMetadata, OAuthClientMetadata, TokenGrantType, TokenPayload,
};
use atrium_xrpc::http::{Method, Request, StatusCode};
use atrium_xrpc::HttpClient;
use elliptic_curve::JwkEcKey;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("no {0:?} endpoint available")]
    NoEndpoint(OAuthEndpointName),
    #[error("unsupported {0:?} authentication method")]
    UnsupportedAuthMethod(OAuthEndpointName),
    #[error(transparent)]
    DpopClient(#[from] crate::http_client::dpop::Error),
    #[error(transparent)]
    Http(#[from] atrium_xrpc::http::Error),
    #[error("http client error: {0}")]
    HttpClient(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("http status: {0:?}")]
    HttpStatus(Option<&'static str>),
    #[error(transparent)]
    SerdeHtmlForm(#[from] serde_html_form::ser::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
pub enum OAuthEndpointName {
    Token,
    Revocation,
    Introspection,
    PushedAuthorizationRequest,
}

impl OAuthEndpointName {}

pub struct OAuthServerAgent {
    server_metadata: OAuthAuthorizationServerMetadata,
    client_metadata: OAuthClientMetadata,
    dpop_client: DpopClient,
}

impl OAuthServerAgent {
    pub fn new(
        dpop_key: JwkEcKey,
        server_metadata: OAuthAuthorizationServerMetadata,
        client_metadata: OAuthClientMetadata,
    ) -> Result<Self> {
        let dpop_client = DpopClient::new(
            dpop_key,
            client_metadata.client_id.clone(),
            server_metadata
                .token_endpoint_auth_signing_alg_values_supported
                .clone(),
        )?;
        Ok(Self {
            server_metadata,
            client_metadata,
            dpop_client,
        })
    }
    pub async fn request<I, O>(&self, endpoint: OAuthEndpointName, payload: I) -> Result<O>
    where
        I: serde::Serialize,
        O: serde::de::DeserializeOwned,
    {
        let Some(url) = self.endpoint(endpoint) else {
            return Err(Error::NoEndpoint(endpoint));
        };
        let body = serde_html_form::to_string(payload)?;
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
        if res.status() == StatusCode::CREATED {
            Ok(serde_json::from_slice(res.body())?)
        } else {
            println!("{}: {}", res.status(), String::from_utf8_lossy(res.body()));
            Err(Error::HttpStatus(res.status().canonical_reason()))
        }
    }
    pub async fn exchange_code(&self, code: &str) -> Result<()> {
        println!(
            "{:?}",
            self.request(
                OAuthEndpointName::Token,
                TokenPayload {
                    grant_type: TokenGrantType::AuthorizationCode,
                    code: code.into(),
                    redirect_uri: self.client_metadata.redirect_uris[0].clone(), // ?
                    client_id: self.client_metadata.client_id.clone(),
                    code_verifier: Some(String::from(
                        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                    )),
                },
            )
            .await?
        );
        Ok(())
    }
    fn build_auth(&self, endpoint: OAuthEndpointName) -> Result<String> {
        if let Some(methods_supported) = self.methods_supprted(endpoint) {
            if let Some(method) = self.client_metadata.token_endpoint_auth_method.as_deref() {
                if !methods_supported.contains(&method.into()) {
                    return Err(Error::UnsupportedAuthMethod(endpoint));
                }
            }
        }
        match self.client_metadata.token_endpoint_auth_method.as_deref() {
            Some("none") => Ok(self.client_metadata.client_id.clone()),
            Some("private_key_jwt") => {
                // TODO
                todo!()
            }
            _ => Err(Error::UnsupportedAuthMethod(endpoint)),
        }
    }
    fn endpoint(&self, endpoint: OAuthEndpointName) -> Option<&String> {
        match endpoint {
            OAuthEndpointName::Token => Some(&self.server_metadata.token_endpoint),
            OAuthEndpointName::Revocation => self.server_metadata.revocation_endpoint.as_ref(),
            OAuthEndpointName::Introspection => {
                self.server_metadata.introspection_endpoint.as_ref()
            }
            OAuthEndpointName::PushedAuthorizationRequest => self
                .server_metadata
                .pushed_authorization_request_endpoint
                .as_ref(),
        }
    }
    fn methods_supprted(&self, endpoint: OAuthEndpointName) -> Option<&Vec<String>> {
        match endpoint {
            OAuthEndpointName::Token | OAuthEndpointName::PushedAuthorizationRequest => self
                .server_metadata
                .token_endpoint_auth_methods_supported
                .as_ref(),
            OAuthEndpointName::Revocation => self
                .server_metadata
                .revocation_endpoint_auth_methods_supported
                .as_ref()
                .or(self
                    .server_metadata
                    .token_endpoint_auth_methods_supported
                    .as_ref()),
            OAuthEndpointName::Introspection => self
                .server_metadata
                .introspection_endpoint_auth_methods_supported
                .as_ref()
                .or(self
                    .server_metadata
                    .token_endpoint_auth_methods_supported
                    .as_ref()),
        }
    }
}
