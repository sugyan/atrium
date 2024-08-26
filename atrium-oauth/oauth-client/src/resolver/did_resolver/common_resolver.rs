use super::super::Result;
use super::base_resolver::{BaseResolver, Method};
use super::plc_resolver::PlcResolver;
use super::web_resolver::WebResolver;
use super::DidResolver;
use std::sync::Arc;

#[derive(Default)]
pub struct CommonResolverConfig<T> {
    pub plc_directory_url: Option<String>,
    pub http_client: Arc<T>,
}

pub struct CommonResolver<T> {
    plc_resolver: Arc<PlcResolver<T>>,
    web_resolver: Arc<WebResolver<T>>,
}

impl<T> CommonResolver<T> {
    pub fn new(config: CommonResolverConfig<T>) -> Result<Self> {
        Ok(Self {
            plc_resolver: Arc::new(PlcResolver::new(
                config.plc_directory_url,
                config.http_client.clone(),
            )?),
            web_resolver: Arc::new(WebResolver::new(config.http_client.clone())),
        })
    }
}

impl<T> BaseResolver for CommonResolver<T>
where
    PlcResolver<T>: DidResolver + Send + Sync + 'static,
    WebResolver<T>: DidResolver + Send + Sync + 'static,
{
    fn get_resolver(&self, method: Method) -> Arc<dyn DidResolver + Send + Sync + 'static> {
        match method {
            Method::Plc => self.plc_resolver.clone(),
            Method::Web => self.web_resolver.clone(),
        }
    }
}
