mod appview_resolver;
mod atproto_resolver;
mod dns_resolver;
mod well_known_resolver;

use super::{Error, Resolver, Result};
pub use appview_resolver::{AppViewHandleResolver, AppViewHandleResolverConfig};
use async_trait::async_trait;
pub use atproto_resolver::{AtprotoHandleResolver, AtprotoHandleResolverConfig};
use atrium_api::types::string::{Did, Handle};
use atrium_xrpc::HttpClient;
pub use dns_resolver::DnsTxtResolver;
use dns_resolver::DynamicDnsTxtResolver;
use std::sync::Arc;
pub use well_known_resolver::{WellKnownHandleResolver, WellKnownHandleResolverConfig};

pub trait HandleResolver: Resolver<Input = Handle, Output = Did> {}

pub struct DynamicHandleResolver {
    resolver: Arc<dyn HandleResolver + Send + Sync + 'static>,
}

impl DynamicHandleResolver {
    pub fn new(resolver: Arc<dyn HandleResolver + Send + Sync + 'static>) -> Self {
        Self { resolver }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Resolver for DynamicHandleResolver {
    type Input = Handle;
    type Output = Did;

    async fn resolve(&self, handle: &Self::Input) -> Result<Self::Output> {
        self.resolver.resolve(handle).await
    }
}

impl HandleResolver for DynamicHandleResolver {}

pub enum HandleResolverImpl {
    Atproto(Arc<dyn DnsTxtResolver + Send + Sync + 'static>),
    AppView(String),
}

pub struct HandleResolverConfig<T> {
    pub r#impl: HandleResolverImpl,
    pub http_client: Arc<T>,
}

impl<T> TryFrom<HandleResolverConfig<T>> for DynamicHandleResolver
where
    T: HttpClient + Send + Sync + 'static,
{
    type Error = Error;

    fn try_from(config: HandleResolverConfig<T>) -> Result<Self> {
        Ok(Self {
            resolver: match config.r#impl {
                HandleResolverImpl::Atproto(dns_txt_resolver) => {
                    Arc::new(AtprotoHandleResolver::new(AtprotoHandleResolverConfig {
                        dns_txt_resolver: DynamicDnsTxtResolver::new(dns_txt_resolver),
                        http_client: config.http_client,
                    })?)
                }
                HandleResolverImpl::AppView(service) => {
                    Arc::new(AppViewHandleResolver::new(AppViewHandleResolverConfig {
                        service_url: service,
                        http_client: config.http_client,
                    })?)
                }
            },
        })
    }
}
