use super::base_resolver::{BaseResolver, Method};
use super::plc_resolver::{PlcDidResolver, PlcDidResolverConfig};
use super::web_resolver::{WebDidResolver, WebDidResolverConfig};
use super::DidResolver;
use crate::error::Result;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct CommonDidResolverConfig<T> {
    pub plc_directory_url: String,
    pub http_client: Arc<T>,
}

pub struct CommonDidResolver<T> {
    plc_resolver: Arc<PlcDidResolver<T>>,
    web_resolver: Arc<WebDidResolver<T>>,
}

impl<T> CommonDidResolver<T> {
    pub fn new(config: CommonDidResolverConfig<T>) -> Result<Self> {
        Ok(Self {
            plc_resolver: Arc::new(PlcDidResolver::new(PlcDidResolverConfig {
                plc_directory_url: config.plc_directory_url,
                http_client: config.http_client.clone(),
            })?),
            web_resolver: Arc::new(WebDidResolver::new(WebDidResolverConfig {
                http_client: config.http_client,
            })),
        })
    }
}

impl<T> BaseResolver for CommonDidResolver<T>
where
    PlcDidResolver<T>: DidResolver + Send + Sync + 'static,
    WebDidResolver<T>: DidResolver + Send + Sync + 'static,
{
    fn get_resolver(&self, method: Method) -> Arc<dyn DidResolver + Send + Sync + 'static> {
        match method {
            Method::Plc => self.plc_resolver.clone(),
            Method::Web => self.web_resolver.clone(),
        }
    }
}
