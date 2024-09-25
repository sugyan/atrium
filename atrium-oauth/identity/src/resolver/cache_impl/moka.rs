use super::super::cached_resolver::{Cache as CacheTrait, CachedResolverConfig};
use moka::{future::Cache, policy::EvictionPolicy};
use std::collections::hash_map::RandomState;
use std::hash::Hash;

pub struct MokaCache<I, O> {
    inner: Cache<I, O, RandomState>,
}

impl<I, O> CacheTrait for MokaCache<I, O>
where
    I: Hash + Eq + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    type Input = I;
    type Output = O;

    fn new(config: CachedResolverConfig) -> Self {
        let mut builder = Cache::<I, O, _>::builder().eviction_policy(EvictionPolicy::lru());
        if let Some(max_capacity) = config.max_capacity {
            builder = builder.max_capacity(max_capacity);
        }
        if let Some(time_to_live) = config.time_to_live {
            builder = builder.time_to_live(time_to_live);
        }
        Self { inner: builder.build() }
    }
    async fn get(&self, key: &Self::Input) -> Option<Self::Output> {
        self.inner.run_pending_tasks().await;
        self.inner.get(key).await
    }
    async fn set(&self, key: Self::Input, value: Self::Output) {
        self.inner.insert(key, value).await;
    }
}
