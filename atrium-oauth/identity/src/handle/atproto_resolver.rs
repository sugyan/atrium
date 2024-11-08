use super::dns_resolver::{DnsHandleResolver, DnsHandleResolverConfig, DnsTxtResolver};
use super::well_known_resolver::{WellKnownHandleResolver, WellKnownHandleResolverConfig};
use super::HandleResolver;
use crate::error::Result;
use crate::Error;
use atrium_api::types::string::{Did, Handle};
use atrium_common::resolver::Resolver;
use atrium_xrpc::HttpClient;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AtprotoHandleResolverConfig<R, T> {
    pub dns_txt_resolver: R,
    pub http_client: Arc<T>,
}

pub struct AtprotoHandleResolver<R, T> {
    dns: DnsHandleResolver<R>,
    http: WellKnownHandleResolver<T>,
}

impl<R, T> AtprotoHandleResolver<R, T> {
    pub fn new(config: AtprotoHandleResolverConfig<R, T>) -> Self {
        Self {
            dns: DnsHandleResolver::new(DnsHandleResolverConfig {
                dns_txt_resolver: config.dns_txt_resolver,
            }),
            http: WellKnownHandleResolver::new(WellKnownHandleResolverConfig {
                http_client: config.http_client,
            }),
        }
    }
}

impl<R, T> Resolver for AtprotoHandleResolver<R, T>
where
    R: DnsTxtResolver + Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
{
    type Input = Handle;
    type Output = Did;
    type Error = Error;

    async fn resolve(&self, handle: &Self::Input) -> Result<Option<Self::Output>> {
        let d_fut = self.dns.resolve(handle);
        let h_fut = self.http.resolve(handle);
        if let Ok(did) = d_fut.await {
            Ok(did)
        } else {
            h_fut.await
        }
    }
}

impl<R, T> HandleResolver for AtprotoHandleResolver<R, T>
where
    R: DnsTxtResolver + Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
{
}
