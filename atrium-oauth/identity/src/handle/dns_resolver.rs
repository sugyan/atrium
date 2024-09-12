use super::HandleResolver;
use crate::error::{Error, Result};
use crate::Resolver;
use async_trait::async_trait;
use atrium_api::types::string::{Did, Handle};
use std::sync::Arc;

const SUBDOMAIN: &str = "_atproto";
const PREFIX: &str = "did=";

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait DnsTxtResolver {
    async fn resolve(
        &self,
        query: &str,
    ) -> core::result::Result<Vec<String>, Box<dyn std::error::Error + Send + Sync + 'static>>;
}

pub struct DynamicDnsTxtResolver {
    resolver: Arc<dyn DnsTxtResolver + Send + Sync + 'static>,
}

impl DynamicDnsTxtResolver {
    pub fn new(resolver: Arc<dyn DnsTxtResolver + Send + Sync + 'static>) -> Self {
        Self { resolver }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl DnsTxtResolver for DynamicDnsTxtResolver {
    async fn resolve(
        &self,
        query: &str,
    ) -> core::result::Result<Vec<String>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.resolver.resolve(query).await
    }
}

#[derive(Clone, Debug)]
pub struct DnsHandleResolverConfig<T> {
    pub dns_txt_resolver: T,
}

pub struct DnsHandleResolver {
    dns_txt_resolver: Arc<dyn DnsTxtResolver + Send + Sync + 'static>,
}

impl DnsHandleResolver {
    pub fn new<T>(config: DnsHandleResolverConfig<T>) -> Result<Self>
    where
        T: DnsTxtResolver + Send + Sync + 'static,
    {
        Ok(Self {
            dns_txt_resolver: Arc::new(config.dns_txt_resolver),
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Resolver for DnsHandleResolver {
    type Input = Handle;
    type Output = Did;

    async fn resolve(&self, handle: &Self::Input) -> Result<Self::Output> {
        for result in self
            .dns_txt_resolver
            .resolve(&format!("{SUBDOMAIN}.{}", handle.as_ref()))
            .await
            .map_err(Error::DnsResolver)?
        {
            if let Some(did) = result.strip_prefix(PREFIX) {
                return did.parse::<Did>().map_err(|e| Error::Did(e.to_string()));
            }
        }
        Err(Error::NotFound)
    }
}

impl HandleResolver for DnsHandleResolver {}
