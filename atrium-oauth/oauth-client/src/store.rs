pub mod memory;
pub mod state;

use async_trait::async_trait;
use std::error::Error;
use std::hash::Hash;

#[async_trait]
pub trait SimpleStore<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    type Error: Error + Send + Sync + 'static;

    async fn get(&self, key: &K) -> Result<Option<V>, Self::Error>;
    async fn set(&self, key: K, value: V) -> Result<Option<V>, Self::Error>;
    async fn del(&self, key: &K) -> Result<Option<V>, Self::Error>;
    async fn clear(&self) -> Result<(), Self::Error>;
}
