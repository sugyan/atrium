use super::cache_impl::CacheImpl;
use crate::error::Result;
use crate::Resolver;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
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

pub struct CachedResolver<R>
where
    R: Resolver,
    R::Input: Sized,
{
    resolver: R,
    cache: CacheImpl<R::Input, R::Output>,
}

impl<R> CachedResolver<R>
where
    R: Resolver,
    R::Input: Sized + Hash + Eq + Send + Sync + 'static,
    R::Output: Clone + Send + Sync + 'static,
{
    pub fn new(resolver: R, config: CachedResolverConfig) -> Self {
        Self { resolver, cache: CacheImpl::new(config) }
    }
}

impl<R> Resolver for CachedResolver<R>
where
    R: Resolver + Send + Sync + 'static,
    R::Input: Clone + Hash + Eq + Send + Sync + 'static + Debug,
    R::Output: Clone + Send + Sync + 'static,
{
    type Input = R::Input;
    type Output = R::Output;

    async fn resolve(&self, input: &Self::Input) -> Result<Self::Output> {
        if let Some(output) = self.cache.get(input).await {
            return Ok(output);
        }
        let output = self.resolver.resolve(input).await?;
        self.cache.set(input.clone(), output.clone()).await;
        Ok(output)
    }
}
