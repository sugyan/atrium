use super::error::{Error, Result};
use crate::http_client;
use async_trait::async_trait;
use atrium_api::types::string::Language;
use atrium_xrpc::http::uri::Builder;
use atrium_xrpc::http::{Request, StatusCode, Uri};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OAuthAuthorizationServerMetadata {
    // https://datatracker.ietf.org/doc/html/rfc8414#section-2
    pub issuer: String,
    pub authorization_endpoint: String, // optional?
    pub token_endpoint: String,         // optional?
    pub jwks_uri: Option<String>,
    pub registration_endpoint: Option<String>,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub response_modes_supported: Option<Vec<String>>,
    pub grant_types_supported: Option<Vec<String>>,
    pub token_endpoint_auth_methods_supported: Option<Vec<String>>,
    pub token_endpoint_auth_signing_alg_values_supported: Option<Vec<String>>,
    pub service_documentation: Option<String>,
    pub ui_locales_supported: Option<Vec<Language>>,
    pub op_policy_uri: Option<String>,
    pub op_tos_uri: Option<String>,
    pub revocation_endpoint: Option<String>,
    pub revocation_endpoint_auth_methods_supported: Option<Vec<String>>,
    pub revocation_endpoint_auth_signing_alg_values_supported: Option<Vec<String>>,
    pub introspection_endpoint: Option<String>,
    pub introspection_endpoint_auth_methods_supported: Option<Vec<String>>,
    pub introspection_endpoint_auth_signing_alg_values_supported: Option<Vec<String>>,
    pub code_challenge_methods_supported: Option<Vec<String>>,

    // https://openid.net/specs/openid-connect-discovery-1_0.html#ProviderMetadata
    pub subject_types_supported: Option<Vec<String>>,
    pub require_request_uri_registration: Option<bool>,

    // https://datatracker.ietf.org/doc/html/rfc9126#section-5
    pub pushed_authorization_request_endpoint: Option<String>,
    pub require_pushed_authorization_requests: Option<bool>,

    // https://datatracker.ietf.org/doc/html/rfc9207#section-3
    pub authorization_response_iss_parameter_supported: Option<bool>,

    // https://datatracker.ietf.org/doc/html/rfc9449#section-5.1
    pub dpop_signing_alg_values_supported: Option<Vec<String>>,

    // https://drafts.aaronpk.com/draft-parecki-oauth-client-id-metadata-document/draft-parecki-oauth-client-id-metadata-document.html#section-5
    pub client_id_metadata_document_supported: Option<bool>,

    // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-resource-metadata-08#name-authorization-server-metada
    pub protected_resources: Option<Vec<String>>,
}

#[async_trait]
pub trait OAuthAuthorizationServerResolver: Send + Sync + 'static {
    async fn get(&self, resource: &str) -> Result<OAuthAuthorizationServerMetadata>;
}

pub struct DefaultOAuthAuthorizationServerResolver;

#[async_trait]
impl OAuthAuthorizationServerResolver for DefaultOAuthAuthorizationServerResolver {
    async fn get(&self, issuer: &str) -> Result<OAuthAuthorizationServerMetadata> {
        let uri = Builder::from(issuer.parse::<Uri>()?)
            .path_and_query("/.well-known/oauth-authorization-server")
            .build()?;
        let client = http_client::get_http_client();
        let res = client
            .send_http(Request::builder().uri(uri).body(Vec::new())?)
            .await
            .map_err(Error::HttpClient)?;
        // https://datatracker.ietf.org/doc/html/rfc8414#section-3.2
        if res.status() == StatusCode::OK {
            let metadata = serde_json::from_slice::<OAuthAuthorizationServerMetadata>(res.body())?;
            // https://datatracker.ietf.org/doc/html/rfc8414#section-3.3
            if metadata.issuer == issuer {
                Ok(metadata)
            } else {
                Err(Error::AuthorizationServerMetadata(format!(
                    "invalid issuer: {}",
                    metadata.issuer
                )))
            }
        } else {
            Err(Error::HttpStatus(res.status().canonical_reason()))
        }
    }
}
