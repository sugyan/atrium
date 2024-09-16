mod cache_impl;
mod cached_resolver;

pub use self::cached_resolver::{CachedResolverConfig, MaybeCachedResolver};
pub use crate::error::Result;
use async_trait::async_trait;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Resolver {
    type Input: ?Sized;
    type Output;

    async fn resolve(&self, input: &Self::Input) -> Result<Self::Output>;
}
