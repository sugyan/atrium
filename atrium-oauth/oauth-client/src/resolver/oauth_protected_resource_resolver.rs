use super::error::{Error, Result};
use super::Resolver;
use async_trait::async_trait;
use atrium_xrpc::http::uri::Builder;
use atrium_xrpc::http::{Request, StatusCode, Uri};
use atrium_xrpc::HttpClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

pub struct DefaultOAuthProtectedResourceResolver<T> {
    http_client: Arc<T>,
}

impl<T> DefaultOAuthProtectedResourceResolver<T> {
    pub fn new(http_client: Arc<T>) -> Self {
        Self { http_client }
    }
}

#[async_trait]
impl<T> Resolver for DefaultOAuthProtectedResourceResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    type Input = String;
    type Output = OAuthProtectedResourceMetadata;

    async fn resolve(&self, resource: &Self::Input) -> Result<Self::Output> {
        let uri = Builder::from(resource.parse::<Uri>()?)
            .path_and_query("/.well-known/oauth-protected-resource")
            .build()?;
        let res = self
            .http_client
            .send_http(Request::builder().uri(uri).body(Vec::new())?)
            .await
            .map_err(Error::HttpClient)?;
        // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-resource-metadata-08#section-3.2
        if res.status() == StatusCode::OK {
            let metadata = serde_json::from_slice::<OAuthProtectedResourceMetadata>(res.body())?;
            // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-resource-metadata-08#section-3.3
            if &metadata.resource == resource {
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
