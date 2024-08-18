use crate::error::{Error, Result};
use crate::resolver::*;
use crate::types::OAuthClientMetadata;
use std::sync::Arc;

#[derive(Debug)]
pub struct OAuthClientConfig {
    // Config
    pub client_metadata: OAuthClientMetadata,
    // Services
    pub handle_resolver: HandleResolverConfig,
    pub plc_directory_url: Option<String>,
}

pub struct OAuthClient {
    resolver: OAuthResolver,
}

impl OAuthClient {
    pub fn new(config: OAuthClientConfig) -> Result<Self> {
        Ok(Self {
            resolver: OAuthResolver::new(IdentityResolver::new(
                Arc::new(
                    CommonResolver::new(CommonResolverConfig {
                        plc_directory_url: config.plc_directory_url,
                    })
                    .map_err(|e| Error::Resolver(crate::resolver::Error::DidResolver(e)))?,
                ),
                Self::handle_resolver(config.handle_resolver),
            )),
        })
    }
}

impl OAuthClient {
    pub async fn authorize(&self, input: impl AsRef<str>) -> Result<()> {
        let (metadata, identity) = self.resolver.resolve(input).await?;
        println!("metadata: {metadata:?}");
        println!("identity: {identity:?}");
        todo!()
    }
    fn handle_resolver(handle_resolver_config: HandleResolverConfig) -> Arc<dyn HandleResolver> {
        match handle_resolver_config {
            HandleResolverConfig::AppView(uri) => Arc::new(AppViewResolver::new(uri)),
            HandleResolverConfig::Service(service) => service,
        }
    }
}
