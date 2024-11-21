pub mod memory;

use std::error::Error;
use std::future::Future;
use std::hash::Hash;

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait Store<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    type Error: Error;

    fn get(&self, key: &K) -> impl Future<Output = Result<Option<V>, Self::Error>>;
    fn set(&self, key: K, value: V) -> impl Future<Output = Result<(), Self::Error>>;
    fn del(&self, key: &K) -> impl Future<Output = Result<(), Self::Error>>;
    fn clear(&self) -> impl Future<Output = Result<(), Self::Error>>;
}
