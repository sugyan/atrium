use super::error::{Error, Result};
use crate::http_client;
use async_trait::async_trait;
use atrium_xrpc::http::uri::Builder;
use atrium_xrpc::http::{Request, StatusCode, Uri};
use serde::{Deserialize, Serialize};

// https://datatracker.ietf.org/doc/draft-ietf-oauth-resource-metadata/
// https://datatracker.ietf.org/doc/html/draft-ietf-oauth-resource-metadata-08#section-2
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OAuthProtectedResourceMetadata {
    pub resource: String,
    pub authorization_servers: Option<Vec<String>>,
    pub jwks_uri: Option<String>,
    pub scopes_supported: Vec<String>,
    pub bearer_methods_supported: Option<Vec<String>>,
    pub resource_signing_alg_values_supported: Option<Vec<String>>,
    pub resource_documentation: Option<String>,
    pub resource_policy_uri: Option<String>,
    pub resource_tos_uri: Option<String>,
}

#[async_trait]
pub trait OAuthProtectedResourceResolver: Send + Sync + 'static {
    async fn get(&self, resource: &str) -> Result<OAuthProtectedResourceMetadata>;
}

pub struct DefaultOAuthProtectedResourceResolver;

#[async_trait]
impl OAuthProtectedResourceResolver for DefaultOAuthProtectedResourceResolver {
    async fn get(&self, resource: &str) -> Result<OAuthProtectedResourceMetadata> {
        let uri = Builder::from(resource.parse::<Uri>()?)
            .path_and_query("/.well-known/oauth-protected-resource")
            .build()?;
        let client = http_client::get_http_client();
        let res = client
            .send_http(Request::builder().uri(uri).body(Vec::new())?)
            .await
            .map_err(Error::HttpClient)?;
        // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-resource-metadata-08#section-3.2
        if res.status() == StatusCode::OK {
            let metadata = serde_json::from_slice::<OAuthProtectedResourceMetadata>(res.body())?;
            // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-resource-metadata-08#section-3.3
            if metadata.resource == resource {
                Ok(metadata)
            } else {
                Err(Error::ProtectedResourceMetadata(format!(
                    "invalid resource: {}",
                    metadata.resource
                )))
            }
        } else {
            Err(Error::HttpStatus(res.status().canonical_reason()))
        }
    }
}
