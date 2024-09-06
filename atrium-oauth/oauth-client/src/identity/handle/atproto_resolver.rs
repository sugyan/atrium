use super::super::{Resolver, Result};
use super::dns_resolver::{DnsHandleResolver, DnsHandleResolverConfig, DnsTxtResolver};
use super::well_known_resolver::{WellKnownHandleResolver, WellKnownHandleResolverConfig};
use super::HandleResolver;
use async_trait::async_trait;
use atrium_api::types::string::{Did, Handle};
use atrium_xrpc::HttpClient;
use futures::future::select_ok;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AtprotoHandleResolverConfig<R, T> {
    pub dns_txt_resolver: R,
    pub http_client: Arc<T>,
}

pub struct AtprotoHandleResolver<T> {
    dns: DnsHandleResolver,
    http: WellKnownHandleResolver<T>,
}

impl<T> AtprotoHandleResolver<T> {
    pub fn new<R>(config: AtprotoHandleResolverConfig<R, T>) -> Result<Self>
    where
        R: DnsTxtResolver + Send + Sync + 'static,
    {
        Ok(Self {
            dns: DnsHandleResolver::new(DnsHandleResolverConfig {
                dns_txt_resolver: config.dns_txt_resolver,
            })?,
            http: WellKnownHandleResolver::new(WellKnownHandleResolverConfig {
                http_client: config.http_client,
            })?,
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T> Resolver for AtprotoHandleResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    type Input = Handle;
    type Output = Did;

    async fn resolve(&self, handle: &Self::Input) -> Result<Self::Output> {
        let (did, _) = select_ok([self.dns.resolve(handle), self.http.resolve(handle)]).await?;
        Ok(did)
    }
}

impl<T> HandleResolver for AtprotoHandleResolver<T> where T: HttpClient + Send + Sync + 'static {}
