use super::error::{Error, Result};
use crate::http_client;
use crate::types::OAuthAuthorizationServerMetadata;
use async_trait::async_trait;
use atrium_xrpc::http::uri::Builder;
use atrium_xrpc::http::{Request, StatusCode, Uri};

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
