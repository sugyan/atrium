mod oauth_authorization_server_resolver;
mod oauth_protected_resource_resolver;

use self::oauth_protected_resource_resolver::DefaultOAuthProtectedResourceResolver;
use crate::identity::{
    DidResolverConfig, Error, HandleResolverConfig, IdentityResolver, IdentityResolverConfig,
    ResolvedIdentity, Resolver, Result,
};
use crate::types::OAuthAuthorizationServerMetadata;
use async_trait::async_trait;
use atrium_xrpc::HttpClient;
use oauth_authorization_server_resolver::DefaultOAuthAuthorizationServerResolver;
use oauth_protected_resource_resolver::OAuthProtectedResourceMetadata;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct OAuthResolverConfig {
    pub did: DidResolverConfig,
    pub handle: HandleResolverConfig,
}

pub struct OAuthResolver<
    T,
    PRR = DefaultOAuthProtectedResourceResolver<T>,
    ASR = DefaultOAuthAuthorizationServerResolver<T>,
> where
    PRR: Resolver<Input = String, Output = OAuthProtectedResourceMetadata>,
    ASR: Resolver<Input = String, Output = OAuthAuthorizationServerMetadata>,
{
    identity_resolver: IdentityResolver<T>,
    protected_resource_resolver: PRR,
    authorization_server_resolver: ASR,
}

impl<T> OAuthResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    pub fn new(config: OAuthResolverConfig, http_client: Arc<T>) -> Result<Self> {
        // TODO: cached resolver?
        let protected_resource_resolver =
            DefaultOAuthProtectedResourceResolver::new(http_client.clone());
        let authorization_server_resolver =
            DefaultOAuthAuthorizationServerResolver::new(http_client.clone());
        Ok(Self {
            identity_resolver: IdentityResolver::new(IdentityResolverConfig {
                did: config.did,
                handle: config.handle,
                http_client,
            })?,
            protected_resource_resolver,
            authorization_server_resolver,
        })
    }
    pub async fn get_authorization_server_metadata(
        &self,
        issuer: impl AsRef<str>,
    ) -> Result<OAuthAuthorizationServerMetadata> {
        self.authorization_server_resolver
            .resolve(&issuer.as_ref().to_string())
            .await
    }
    async fn resolve_from_service(&self, input: &str) -> Result<OAuthAuthorizationServerMetadata> {
        // Assume first that input is a PDS URL (as required by ATPROTO)
        if let Ok(metadata) = self.get_resource_server_metadata(input).await {
            return Ok(metadata);
        }
        // Fallback to trying to fetch as an issuer (Entryway)
        self.get_authorization_server_metadata(input).await
    }
    pub(crate) async fn resolve_from_identity(
        &self,
        input: &str,
    ) -> Result<(OAuthAuthorizationServerMetadata, ResolvedIdentity)> {
        let identity = self.identity_resolver.resolve(input).await?;
        let metadata = self.get_resource_server_metadata(&identity.pds).await?;
        Ok((metadata, identity))
    }
    async fn get_resource_server_metadata(
        &self,
        pds: &str,
    ) -> Result<OAuthAuthorizationServerMetadata> {
        let rs_metadata = self
            .protected_resource_resolver
            .resolve(&pds.to_string())
            .await?;
        // ATPROTO requires one, and only one, authorization server entry
        // > That document MUST contain a single item in the authorization_servers array.
        // https://github.com/bluesky-social/proposals/tree/main/0004-oauth#server-metadata
        let issuer = match &rs_metadata.authorization_servers {
            Some(servers) if !servers.is_empty() => {
                if servers.len() > 1 {
                    return Err(Error::ProtectedResourceMetadata(format!(
                        "unable to determine authorization server for PDS: {pds}"
                    )));
                }
                &servers[0]
            }
            _ => {
                return Err(Error::ProtectedResourceMetadata(format!(
                    "no authorization server found for PDS: {pds}"
                )))
            }
        };
        let as_metadata = self.get_authorization_server_metadata(issuer).await?;
        // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-resource-metadata-08#name-authorization-server-metada
        if let Some(protected_resources) = &as_metadata.protected_resources {
            if !protected_resources.contains(&rs_metadata.resource) {
                return Err(Error::AuthorizationServerMetadata(format!(
                    "pds {pds} does not protected by issuer: {issuer}",
                )));
            }
        }

        // TODO: atproot specific validation?
        // https://github.com/bluesky-social/proposals/tree/main/0004-oauth#server-metadata
        //
        // eg.
        // https://drafts.aaronpk.com/draft-parecki-oauth-client-id-metadata-document/draft-parecki-oauth-client-id-metadata-document.html
        // if as_metadata.client_id_metadata_document_supported != Some(true) {
        //     return Err(Error::AuthorizationServerMetadata(format!(
        //         "authorization server does not support client_id_metadata_document: {issuer}"
        //     )));
        // }

        Ok(as_metadata)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T> Resolver for OAuthResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    type Input = str;
    type Output = (OAuthAuthorizationServerMetadata, Option<ResolvedIdentity>);

    async fn resolve(&self, input: &Self::Input) -> Result<Self::Output> {
        // Allow using an entryway, or PDS url, directly as login input (e.g.
        // when the user forgot their handle, or when the handle does not
        // resolve to a DID)
        Ok(if input.starts_with("https://") {
            (self.resolve_from_service(input.as_ref()).await?, None)
        } else {
            let (metadata, identity) = self.resolve_from_identity(input).await?;
            (metadata, Some(identity))
        })
    }
}
