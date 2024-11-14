use super::{CellStore, MapStore};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("memory store error")]
pub struct Error;

#[derive(Clone)]
pub struct MemoryCellStore<V> {
    store: Arc<Mutex<Option<V>>>,
}

impl<V> Default for MemoryCellStore<V> {
    fn default() -> Self {
        Self { store: Arc::new(Mutex::new(None)) }
    }
}

impl<V> CellStore<V> for MemoryCellStore<V>
where
    V: Debug + Clone + Send + Sync + 'static,
{
    type Error = Error;

    async fn get(&self) -> Result<Option<V>, Self::Error> {
        Ok((*self.store.lock().unwrap()).clone())
    }
    async fn set(&self, value: V) -> Result<(), Self::Error> {
        *self.store.lock().unwrap() = Some(value);
        Ok(())
    }
    async fn clear(&self) -> Result<(), Self::Error> {
        *self.store.lock().unwrap() = None;
        Ok(())
    }
}

// TODO: LRU cache?
#[derive(Clone)]
pub struct MemoryMapStore<K, V> {
    store: Arc<Mutex<HashMap<K, V>>>,
}

impl<K, V> Default for MemoryMapStore<K, V> {
    fn default() -> Self {
        Self { store: Arc::new(Mutex::new(HashMap::new())) }
    }
}

impl<K, V> MapStore<K, V> for MemoryMapStore<K, V>
where
    K: Debug + Eq + Hash + Send + Sync + 'static,
    V: Debug + Clone + Send + Sync + 'static,
{
    type Error = Error;

    async fn get(&self, key: &K) -> Result<Option<V>, Self::Error> {
        Ok(self.store.lock().unwrap().get(key).cloned())
    }
    async fn set(&self, key: K, value: V) -> Result<(), Self::Error> {
        self.store.lock().unwrap().insert(key, value);
        Ok(())
    }
    async fn del(&self, key: &K) -> Result<(), Self::Error> {
        self.store.lock().unwrap().remove(key);
        Ok(())
    }
    async fn clear(&self) -> Result<(), Self::Error> {
        self.store.lock().unwrap().clear();
        Ok(())
    }
}
