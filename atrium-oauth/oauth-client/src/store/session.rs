use atrium_api::types::string::Did;

use crate::session::Session;

use super::SimpleStore;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

pub struct SessionStore<K = Did, V = Session> {
    store: Arc<RwLock<HashMap<K, V>>>,
}

impl<K, V> Default for SessionStore<K, V> {
    fn default() -> Self {
        Self { store: Arc::new(RwLock::new(HashMap::new())) }
    }
}

impl<K, V> SimpleStore<K, V> for SessionStore<K, V>
where
    K: Debug + Eq + Hash + Send + Sync + 'static,
    V: Debug + Clone + Send + Sync + 'static,
{
    type Error = Infallible;

    async fn get(&self, key: &K) -> Result<Option<V>, Self::Error> {
        Ok(self.store.read().expect("lock should never be poisoned").get(key).cloned())
    }
    async fn set(&self, key: K, value: V) -> Result<(), Self::Error> {
        self.store.write().expect("lock should never be poisoned").insert(key, value);
        Ok(())
    }
    async fn del(&self, key: &K) -> Result<(), Self::Error> {
        self.store.write().expect("lock should never be poisoned").remove(key);
        Ok(())
    }
    async fn clear(&self) -> Result<(), Self::Error> {
        self.store.write().expect("lock should never be poisoned").clear();
        Ok(())
    }
}
