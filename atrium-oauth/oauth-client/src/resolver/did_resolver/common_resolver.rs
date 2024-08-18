use super::base_resolver::{BaseResolver, Method};
use super::plc_resolver::PlcResolver;
use super::web_resolver::WebResolver;
use super::{DidResolver, Result};
use std::sync::Arc;

#[derive(Default)]
pub struct CommonResolverConfig {
    pub plc_directory_url: Option<String>,
}

pub struct CommonResolver {
    plc_resolver: Arc<PlcResolver>,
    web_resolver: Arc<WebResolver>,
}

impl CommonResolver {
    pub fn new(config: CommonResolverConfig) -> Result<Self> {
        Ok(Self {
            plc_resolver: Arc::new(PlcResolver::new(config.plc_directory_url)?),
            web_resolver: Arc::new(WebResolver::new()),
        })
    }
}

impl BaseResolver for CommonResolver {
    fn get_resolver(&self, method: Method) -> Arc<dyn DidResolver + Send + Sync + 'static> {
        match method {
            Method::Plc => self.plc_resolver.clone(),
            Method::Web => self.web_resolver.clone(),
        }
    }
}
