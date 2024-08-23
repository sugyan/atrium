use super::error::{Error, Result};
use crate::types::OAuthAuthorizationServerMetadata;
use async_trait::async_trait;
use atrium_xrpc::http::uri::Builder;
use atrium_xrpc::http::{Request, StatusCode, Uri};
use atrium_xrpc::HttpClient;
use std::sync::Arc;

#[async_trait]
pub trait OAuthAuthorizationServerResolver {
    async fn get(&self, resource: &str) -> Result<OAuthAuthorizationServerMetadata>;
}

pub struct DefaultOAuthAuthorizationServerResolver<T> {
    http_client: Arc<T>,
}

impl<T> DefaultOAuthAuthorizationServerResolver<T> {
    pub fn new(http_client: Arc<T>) -> Self {
        Self { http_client }
    }
}

#[async_trait]
impl<T> OAuthAuthorizationServerResolver for DefaultOAuthAuthorizationServerResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    async fn get(&self, issuer: &str) -> Result<OAuthAuthorizationServerMetadata> {
        let uri = Builder::from(issuer.parse::<Uri>()?)
            .path_and_query("/.well-known/oauth-authorization-server")
            .build()?;
        let res = self
            .http_client
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
