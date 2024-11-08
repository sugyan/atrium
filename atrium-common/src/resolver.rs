mod cached;
mod error;
mod throttled;

pub use self::cached::CachedResolver;
pub use self::error::{Error, Result};
pub use self::throttled::ThrottledResolver;
pub use super::types::cached::r#impl::CacheImpl;
use std::future::Future;

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait Resolver {
    type Input: ?Sized;
    type Output;
    type Error;

    fn resolve(
        &self,
        input: &Self::Input,
    ) -> impl Future<Output = core::result::Result<Option<Self::Output>, Self::Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::cached::r#impl::Cache;
    use crate::types::cached::{CacheConfig, Cacheable};
    use crate::types::throttled::Throttleable;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::RwLock;
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    #[cfg(not(target_arch = "wasm32"))]
    async fn sleep(duration: Duration) {
        tokio::time::sleep(duration).await;
    }

    #[cfg(target_arch = "wasm32")]
    async fn sleep(duration: Duration) {
        gloo_timers::future::sleep(duration).await;
    }

    struct MockResolver {
        data: HashMap<String, String>,
        counts: Arc<RwLock<HashMap<String, usize>>>,
    }

    impl Resolver for MockResolver {
        type Input = String;
        type Output = String;
        type Error = Error;

        async fn resolve(&self, input: &Self::Input) -> Result<Option<Self::Output>> {
            sleep(Duration::from_millis(10)).await;
            *self.counts.write().await.entry(input.clone()).or_default() += 1;
            Ok(self.data.get(input).cloned())
        }
    }

    fn mock_resolver(counts: Arc<RwLock<HashMap<String, usize>>>) -> MockResolver {
        MockResolver {
            data: [
                (String::from("k1"), String::from("v1")),
                (String::from("k2"), String::from("v2")),
            ]
            .into_iter()
            .collect(),
            counts,
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    async fn test_no_cached() {
        let counts = Arc::new(RwLock::new(HashMap::new()));
        let resolver = mock_resolver(counts.clone());
        for (input, expected) in [
            ("k1", Some("v1")),
            ("k2", Some("v2")),
            ("k2", Some("v2")),
            ("k1", Some("v1")),
            ("k3", None),
            ("k1", Some("v1")),
            ("k3", None),
        ] {
            let result = resolver.resolve(&input.to_string()).await;
            match expected {
                Some(value) => {
                    assert_eq!(result.expect("failed to resolve").as_deref(), Some(value))
                }
                None => {
                    assert_eq!(result.expect("failed to resolve").as_deref(), None)
                }
            }
        }
        assert_eq!(
            *counts.read().await,
            [(String::from("k1"), 3), (String::from("k2"), 2), (String::from("k3"), 2),]
                .into_iter()
                .collect()
        );
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    async fn test_cached() {
        let counts = Arc::new(RwLock::new(HashMap::new()));
        let resolver = mock_resolver(counts.clone()).cached(CacheImpl::new(CacheConfig::default()));
        for (input, expected) in [
            ("k1", Some("v1")),
            ("k2", Some("v2")),
            ("k2", Some("v2")),
            ("k1", Some("v1")),
            ("k3", None),
            ("k1", Some("v1")),
            ("k3", None),
        ] {
            let result = resolver.resolve(&input.to_string()).await;
            match expected {
                Some(value) => {
                    assert_eq!(result.expect("failed to resolve").as_deref(), Some(value))
                }
                None => {
                    assert_eq!(result.expect("failed to resolve").as_deref(), None)
                }
            }
        }
        assert_eq!(
            *counts.read().await,
            [(String::from("k1"), 1), (String::from("k2"), 1), (String::from("k3"), 2),]
                .into_iter()
                .collect()
        );
    }

    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_cached_with_max_capacity() {
        let counts = Arc::new(RwLock::new(HashMap::new()));
        let resolver = mock_resolver(counts.clone())
            .cached(CacheImpl::new(CacheConfig { max_capacity: Some(1), ..Default::default() }));
        for (input, expected) in [
            ("k1", Some("v1")),
            ("k2", Some("v2")),
            ("k2", Some("v2")),
            ("k1", Some("v1")),
            ("k3", None),
            ("k1", Some("v1")),
            ("k3", None),
        ] {
            let result = resolver.resolve(&input.to_string()).await;
            match expected {
                Some(value) => {
                    assert_eq!(result.expect("failed to resolve").as_deref(), Some(value))
                }
                None => {
                    assert_eq!(result.expect("failed to resolve").as_deref(), None)
                }
            }
        }
        assert_eq!(
            *counts.read().await,
            [(String::from("k1"), 2), (String::from("k2"), 1), (String::from("k3"), 2),]
                .into_iter()
                .collect()
        );
    }

    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_cached_with_time_to_live() {
        let counts = Arc::new(RwLock::new(HashMap::new()));
        let resolver = mock_resolver(counts.clone()).cached(CacheImpl::new(CacheConfig {
            time_to_live: Some(Duration::from_millis(10)),
            ..Default::default()
        }));
        for _ in 0..10 {
            let result = resolver.resolve(&String::from("k1")).await;
            assert_eq!(result.expect("failed to resolve").as_deref(), Some("v1"));
        }
        sleep(Duration::from_millis(10)).await;
        for _ in 0..10 {
            let result = resolver.resolve(&String::from("k1")).await;
            assert_eq!(result.expect("failed to resolve").as_deref(), Some("v1"));
        }
        assert_eq!(*counts.read().await, [(String::from("k1"), 2)].into_iter().collect());
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    async fn test_throttled() {
        let counts = Arc::new(RwLock::new(HashMap::new()));
        let resolver = Arc::new(mock_resolver(counts.clone()).throttled());

        let mut handles = Vec::new();
        for (input, expected) in [
            ("k1", Some("v1")),
            ("k2", Some("v2")),
            ("k2", Some("v2")),
            ("k1", Some("v1")),
            ("k3", None),
            ("k1", Some("v1")),
            ("k3", None),
        ] {
            let resolver = resolver.clone();
            handles.push(async move { (resolver.resolve(&input.to_string()).await, expected) });
        }
        for (result, expected) in futures::future::join_all(handles).await {
            match expected {
                Some(value) => {
                    assert_eq!(result.expect("failed to resolve").as_deref(), Some(value))
                }
                None => {
                    assert_eq!(result.expect("failed to resolve").as_deref(), None)
                }
            }
        }
        assert_eq!(
            *counts.read().await,
            [(String::from("k1"), 1), (String::from("k2"), 1), (String::from("k3"), 1),]
                .into_iter()
                .collect()
        );
    }
}
