use super::{OAuthServerAgent, Result};
use crate::{
    keyset::Keyset,
    resolver::OAuthResolver,
    types::{OAuthAuthorizationServerMetadata, OAuthClientMetadata},
};
use atrium_identity::{did::DidResolver, handle::HandleResolver};
use atrium_xrpc::HttpClient;
use jose_jwk::Key;
use std::sync::Arc;

pub struct OAuthServerFactory<T, D, H>
where
    T: HttpClient + Send + Sync + 'static,
{
    client_metadata: OAuthClientMetadata,
    resolver: Arc<OAuthResolver<T, D, H>>,
    http_client: Arc<T>,
    keyset: Option<Keyset>,
}

impl<T, D, H> OAuthServerFactory<T, D, H>
where
    T: HttpClient + Send + Sync + 'static,
{
    pub fn new(
        client_metadata: OAuthClientMetadata,
        resolver: Arc<OAuthResolver<T, D, H>>,
        http_client: Arc<T>,
        keyset: Option<Keyset>,
    ) -> Self {
        OAuthServerFactory { client_metadata, resolver, http_client, keyset }
    }
}

impl<T, D, H> OAuthServerFactory<T, D, H>
where
    T: HttpClient + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
{
    pub async fn build_from_issuer(
        &self,
        dpop_key: Key,
        issuer: impl AsRef<str>,
    ) -> Result<OAuthServerAgent<T, D, H>> {
        let server_metadata = self.resolver.get_authorization_server_metadata(&issuer).await?;
        self.build_from_metadata(dpop_key, server_metadata)
    }
    pub fn build_from_metadata(
        &self,
        dpop_key: Key,
        server_metadata: OAuthAuthorizationServerMetadata,
    ) -> Result<OAuthServerAgent<T, D, H>> {
        OAuthServerAgent::new(
            dpop_key,
            server_metadata,
            self.client_metadata.clone(),
            Arc::clone(&self.resolver),
            Arc::clone(&self.http_client),
            self.keyset.clone(),
        )
    }
}
