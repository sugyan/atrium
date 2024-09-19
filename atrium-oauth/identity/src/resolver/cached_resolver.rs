use super::cache_impl::CacheImpl;
use crate::error::Result;
use crate::Resolver;
use async_trait::async_trait;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub(crate) trait Cache {
    type Input: Hash + Eq + Sync + 'static;
    type Output: Clone + Sync + 'static;

    fn new(config: CachedResolverConfig) -> Self;
    async fn get(&self, key: &Self::Input) -> Option<Self::Output>;
    async fn set(&self, key: Self::Input, value: Self::Output);
}

#[derive(Clone, Debug, Default)]
pub struct CachedResolverConfig {
    pub max_capacity: Option<u64>,
    pub time_to_live: Option<Duration>,
}

pub struct MaybeCachedResolver<R, I, O>
where
    R: Resolver<Input = I, Output = O>,
{
    resolver: R,
    cache: Option<CacheImpl<I, O>>,
}

impl<R, I, O> MaybeCachedResolver<R, I, O>
where
    R: Resolver<Input = I, Output = O>,
    I: Hash + Eq + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    pub fn new(resolver: R, config: Option<CachedResolverConfig>) -> Self {
        let cache = config.map(CacheImpl::new);
        Self { resolver, cache }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<R, I, O> Resolver for MaybeCachedResolver<R, I, O>
where
    R: Resolver<Input = I, Output = O> + Send + Sync + 'static,
    I: Clone + Hash + Eq + Send + Sync + 'static + Debug,
    O: Clone + Send + Sync + 'static,
{
    type Input = R::Input;
    type Output = R::Output;

    async fn resolve(&self, input: &Self::Input) -> Result<Self::Output> {
        if let Some(cache) = &self.cache {
            if let Some(output) = cache.get(input).await {
                return Ok(output);
            }
        }
        let output = self.resolver.resolve(input).await?;
        if let Some(cache) = &self.cache {
            cache.set(input.clone(), output.clone()).await;
        }
        Ok(output)
    }
}
