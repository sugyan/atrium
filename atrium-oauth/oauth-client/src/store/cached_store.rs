use std::hash::Hash;
use std::marker::{Send, Sync};

use atrium_api::types::string::Did;

use crate::session::Session;

use super::session::SessionStore;
use super::SimpleStore;

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait Cache<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    async fn get(&self, key: &K, config: CachedStoreConfig) -> Option<V>;

    async fn get_stored(&self, key: &K, config: CachedStoreConfig) -> Option<V>;

    async fn set_stored(&self, key: K, value: V);

    async fn del_stored(&self, key: &K);

    // fn bind(
    //     &self,
    //     key: K,
    // ) -> impl FnOnce(CachedStoreConfig) -> Pin<Box<dyn Future<Output = Option<V>>>>;
}

pub trait SessionCache: Cache<Did, Session> {}

impl SessionCache for CachedStore<SessionStore> {}

#[derive(Clone, Debug, Default)]
pub struct CachedStoreConfig {
    pub no_cache: bool,
    pub allow_stale: bool,
}

pub enum PendingEntry<T> {
    Fresh(T),
    Stale(T),
}

pub struct CachedStore<S> {
    pub store: S,
    pub config: CachedStoreConfig,
    // pending_entries: HashMap<K, PendingEntry<V>>,
}

impl<S> CachedStore<S> {
    pub fn new(store: S, config: CachedStoreConfig) -> Self {
        Self { store, config }
    }
}

impl<S> Default for CachedStore<S>
where
    S: Default,
{
    fn default() -> Self {
        Self { store: Default::default(), config: Default::default() }
    }
}

impl<S, K, V> Cache<K, V> for CachedStore<S>
where
    S: SimpleStore<K, V> + Sync,
    K: Eq + Hash + Send + Sync,
    V: Clone + Send,
{
    async fn get(&self, _key: &K, _config: CachedStoreConfig) -> Option<V> {
        todo!()
    }

    // fn bind(
    //     &self,
    //     key: K,
    // ) -> impl FnOnce(CachedStoreConfig) -> Pin<Box<dyn Future<Output = Option<V>>>> {
    //     move |config| Box::pin(self.get(&key, config))
    // }

    async fn get_stored(&self, key: &K, _config: CachedStoreConfig) -> Option<V> {
        self.store.get(key).await.expect("todo")
    }

    async fn set_stored(&self, key: K, value: V) {
        self.store.set(key, value).await.expect("todo")
    }

    async fn del_stored(&self, key: &K) {
        self.store.del(key).await.expect("todo")
    }
}
