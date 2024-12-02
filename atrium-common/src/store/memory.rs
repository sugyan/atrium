use super::Store;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
#[error("memory store error")]
pub struct Error;

#[derive(Clone)]
pub struct MemoryStore<K, V> {
    store: Arc<Mutex<HashMap<K, V>>>,
}

impl<K, V> Default for MemoryStore<K, V> {
    fn default() -> Self {
        Self { store: Arc::new(Mutex::new(HashMap::new())) }
    }
}

impl<K, V> Store<K, V> for MemoryStore<K, V>
where
    K: Debug + Eq + Hash + Send + Sync + 'static,
    V: Debug + Clone + Send + Sync + 'static,
{
    type Error = Error;

    async fn get(&self, key: &K) -> Result<Option<V>, Self::Error> {
        Ok(self.store.lock().await.get(key).cloned())
    }
    async fn set(&self, key: K, value: V) -> Result<(), Self::Error> {
        self.store.lock().await.insert(key, value);
        Ok(())
    }
    async fn del(&self, key: &K) -> Result<(), Self::Error> {
        self.store.lock().await.remove(key);
        Ok(())
    }
    async fn clear(&self) -> Result<(), Self::Error> {
        self.store.lock().await.clear();
        Ok(())
    }
}
