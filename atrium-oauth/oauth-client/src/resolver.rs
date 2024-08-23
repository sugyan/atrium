mod did_resolver;
mod error;
mod handle_resolver;
mod identity_resolver;
mod oauth_authorization_server_resolver;
mod oauth_protected_resource_resolver;

pub use self::did_resolver::{CommonResolver, CommonResolverConfig, DidResolver};
pub use self::error::{Error, Result};
pub use self::handle_resolver::{AppViewResolver, HandleResolver, HandleResolverConfig};
pub use self::identity_resolver::IdentityResolver;
use self::oauth_protected_resource_resolver::{
    DefaultOAuthProtectedResourceResolver, OAuthProtectedResourceResolver,
};
use crate::types::OAuthAuthorizationServerMetadata;
use atrium_xrpc::HttpClient;
use identity_resolver::ResolvedIdentity;
use oauth_authorization_server_resolver::{
    DefaultOAuthAuthorizationServerResolver, OAuthAuthorizationServerResolver,
};
use std::marker::PhantomData;
use std::sync::Arc;

pub struct OAuthResolver<
    T,
    PRR = DefaultOAuthProtectedResourceResolver<T>,
    ASR = DefaultOAuthAuthorizationServerResolver<T>,
> where
    PRR: OAuthProtectedResourceResolver,
    ASR: OAuthAuthorizationServerResolver,
{
    identity_resolver: IdentityResolver,
    protected_resource_resolver: PRR,
    authorization_server_resolver: ASR,
    _phantom: PhantomData<T>,
}

impl<T> OAuthResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    pub fn new(identity_resolver: IdentityResolver, http_client: Arc<T>) -> Self {
        // TODO: cached resolver?
        Self {
            identity_resolver,
            protected_resource_resolver: DefaultOAuthProtectedResourceResolver::new(
                http_client.clone(),
            ),
            authorization_server_resolver: DefaultOAuthAuthorizationServerResolver::new(
                http_client.clone(),
            ),
            _phantom: PhantomData,
        }
    }
    pub async fn resolve(
        &self,
        input: impl AsRef<str>,
    ) -> Result<(OAuthAuthorizationServerMetadata, Option<ResolvedIdentity>)> {
        // TODO: entryway, or PDS url
        let (metadata, identity) = self.resolve_from_identity(input.as_ref()).await?;
        Ok((metadata, Some(identity)))
    }
    pub async fn get_authorization_server_metadata(
        &self,
        issuer: impl AsRef<str>,
    ) -> Result<OAuthAuthorizationServerMetadata> {
        self.authorization_server_resolver
            .get(issuer.as_ref())
            .await
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
        let rs_metadata = self.protected_resource_resolver.get(pds).await?;
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
