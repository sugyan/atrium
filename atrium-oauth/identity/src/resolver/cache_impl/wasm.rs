use super::super::cached_resolver::{Cache as CacheTrait, CachedResolverConfig};
use lru::LruCache;
use std::collections::HashMap;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::Mutex;
use web_time::{Duration, Instant};

enum Store<I, O> {
    Lru(LruCache<I, O>),
    HashMap(HashMap<I, O>),
}

impl<I, O> Store<I, O>
where
    I: Hash + Eq + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    fn get(&mut self, key: &I) -> Option<O> {
        match self {
            Self::Lru(cache) => cache.get(key).cloned(),
            Self::HashMap(map) => map.get(key).cloned(),
        }
    }
    fn set(&mut self, key: I, value: O) {
        match self {
            Self::Lru(cache) => {
                cache.put(key, value);
            }
            Self::HashMap(map) => {
                map.insert(key, value);
            }
        }
    }
    fn del(&mut self, key: &I) {
        match self {
            Self::Lru(cache) => {
                cache.pop(key);
            }
            Self::HashMap(map) => {
                map.remove(key);
            }
        }
    }
}

#[derive(Clone, Debug)]
struct ValueWithInstant<O> {
    value: O,
    instant: Instant,
}

pub struct WasmCache<I, O> {
    inner: Arc<Mutex<Store<I, ValueWithInstant<O>>>>,
    expiration: Option<Duration>,
}

impl<I, O> CacheTrait for WasmCache<I, O>
where
    I: Hash + Eq + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    type Input = I;
    type Output = O;

    fn new(config: CachedResolverConfig) -> Self {
        let store = if let Some(max_capacity) = config.max_capacity {
            Store::Lru(LruCache::new(
                NonZeroUsize::new(max_capacity as usize)
                    .expect("max_capacity must be greater than 0"),
            ))
        } else {
            Store::HashMap(HashMap::new())
        };
        Self { inner: Arc::new(Mutex::new(store)), expiration: config.time_to_live }
    }
    async fn get(&self, key: &Self::Input) -> Option<Self::Output> {
        let mut cache = self.inner.lock().await;
        if let Some(ValueWithInstant { value, instant }) = cache.get(key) {
            if let Some(expiration) = self.expiration {
                if instant.elapsed() > expiration {
                    cache.del(key);
                    return None;
                }
            }
            Some(value)
        } else {
            None
        }
    }
    async fn set(&self, key: Self::Input, value: Self::Output) {
        self.inner.lock().await.set(key, ValueWithInstant { value, instant: Instant::now() });
    }
}
