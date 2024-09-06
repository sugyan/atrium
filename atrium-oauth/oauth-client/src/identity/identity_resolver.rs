use super::did::{CommonDidResolver, CommonDidResolverConfig};
use super::error::{Error, Result};
use super::handle::{DynamicHandleResolver, HandleResolverImpl};
use super::Resolver;
use async_trait::async_trait;
use atrium_api::types::string::AtIdentifier;
use atrium_xrpc::HttpClient;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedIdentity {
    pub did: String,
    pub pds: String,
}

#[derive(Clone, Debug, Default)]
pub struct DidResolverConfig {
    pub plc_directory_url: Option<String>,
}

pub struct HandleResolverConfig {
    pub r#impl: HandleResolverImpl,
}

pub struct IdentityResolverConfig<T> {
    pub did: DidResolverConfig,
    pub handle: HandleResolverConfig,
    pub http_client: Arc<T>,
}

pub struct IdentityResolver<T, D = CommonDidResolver<T>, H = DynamicHandleResolver> {
    did_resolver: D,
    handle_resolver: H,
    _phantom: PhantomData<T>,
}

impl<T> IdentityResolver<T> {
    pub fn new(config: IdentityResolverConfig<T>) -> Result<Self>
    where
        T: HttpClient + Send + Sync + 'static,
    {
        Ok(Self::from((
            CommonDidResolver::new(CommonDidResolverConfig {
                plc_directory_url: config.did.plc_directory_url,
                http_client: config.http_client.clone(),
            })?,
            DynamicHandleResolver::try_from(super::handle::HandleResolverConfig {
                r#impl: config.handle.r#impl,
                http_client: config.http_client,
            })?,
        )))
    }
}

impl<T, D, H> From<(D, H)> for IdentityResolver<T, D, H> {
    fn from((did_resolver, handle_resolver): (D, H)) -> Self {
        Self {
            did_resolver,
            handle_resolver,
            _phantom: PhantomData,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T> Resolver for IdentityResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    type Input = str;
    type Output = ResolvedIdentity;

    async fn resolve(&self, input: &Self::Input) -> Result<Self::Output> {
        let document = match input
            .parse::<AtIdentifier>()
            .map_err(|e| Error::AtIdentifier(e.to_string()))?
        {
            AtIdentifier::Did(did) => self.did_resolver.resolve(&did).await?,
            AtIdentifier::Handle(handle) => {
                let did = self.handle_resolver.resolve(&handle).await?;
                let document = self.did_resolver.resolve(&did).await?;
                if let Some(aka) = &document.also_known_as {
                    if !aka.contains(&format!("at://{}", handle.as_str())) {
                        return Err(Error::DidDocument(format!(
                            "did document for `{}` does not include the handle `{}`",
                            did.as_str(),
                            handle.as_str()
                        )));
                    }
                }
                document
            }
        };
        let Some(service) = document.get_pds_endpoint() else {
            return Err(Error::DidDocument(format!(
                "no valid `AtprotoPersonalDataServer` service found in `{}`",
                document.id
            )));
        };
        Ok(ResolvedIdentity {
            did: document.id,
            pds: service,
        })
    }
}
