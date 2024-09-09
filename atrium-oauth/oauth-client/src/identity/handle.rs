mod appview_resolver;
mod atproto_resolver;
mod dns_resolver;
#[cfg(feature = "doh-handle-resolver")]
mod doh_dns_txt_resolver;
mod well_known_resolver;

pub use self::appview_resolver::{AppViewHandleResolver, AppViewHandleResolverConfig};
pub use self::atproto_resolver::{AtprotoHandleResolver, AtprotoHandleResolverConfig};
pub use self::dns_resolver::DnsTxtResolver;
#[cfg(feature = "doh-handle-resolver")]
pub use self::doh_dns_txt_resolver::{DohDnsTxtResolver, DohDnsTxtResolverConfig};
pub use self::well_known_resolver::{WellKnownHandleResolver, WellKnownHandleResolverConfig};
use super::{Error, Resolver, Result};
use async_trait::async_trait;
use atrium_api::types::string::{Did, Handle};
use atrium_xrpc::HttpClient;
use dns_resolver::DynamicDnsTxtResolver;
use std::sync::Arc;

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

#[derive(Clone)]
pub enum HandleResolverImpl {
    Atproto(Arc<dyn DnsTxtResolver + Send + Sync + 'static>),
    AppView(String),
}

impl std::fmt::Debug for HandleResolverImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandleResolverImpl::Atproto(_) => write!(f, "Atproto"),
            HandleResolverImpl::AppView(url) => write!(f, "AppView({url})"),
        }
    }
}

#[derive(Clone, Debug)]
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
