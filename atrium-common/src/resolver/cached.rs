use std::hash::Hash;

use crate::types::cached::r#impl::{Cache, CacheImpl};
use crate::types::cached::Cached;

use super::Resolver;

pub type CachedResolver<R> = Cached<R, CacheImpl<<R as Resolver>::Input, <R as Resolver>::Output>>;

impl<R, C> Resolver for Cached<R, C>
where
    R: Resolver + Send + Sync + 'static,
    R::Input: Clone + Hash + Eq + Send + Sync + 'static,
    R::Output: Clone + Send + Sync + 'static,
    C: Cache<Input = R::Input, Output = R::Output> + Send + Sync + 'static,
    C::Input: Clone + Hash + Eq + Send + Sync + 'static,
    C::Output: Clone + Send + Sync + 'static,
{
    type Input = R::Input;
    type Output = R::Output;
    type Error = R::Error;

    async fn resolve(&self, input: &Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        match self.cache.get(input).await {
            Some(cached) => Ok(Some(cached)),
            None => {
                let result = self.inner.resolve(input).await?;

                if let Some(result) = result.as_ref().cloned() {
                    self.cache.set(input.clone(), result.clone()).await;
                }
                Ok(result)
            }
        }
    }
}
