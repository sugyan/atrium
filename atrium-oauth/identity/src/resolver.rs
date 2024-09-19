mod cache_impl;
mod cached_resolver;
mod throttled_resolver;

pub use self::cached_resolver::{CachedResolverConfig, MaybeCachedResolver};
pub use self::throttled_resolver::ThrottledResolver;
pub use crate::error::Result;
use async_trait::async_trait;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Resolver {
    type Input: ?Sized;
    type Output;

    async fn resolve(&self, input: &Self::Input) -> Result<Self::Output>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
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

    #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl Resolver for MockResolver {
        type Input = String;
        type Output = String;

        async fn resolve(&self, input: &Self::Input) -> Result<Self::Output> {
            sleep(Duration::from_millis(10)).await;
            *self.counts.write().await.entry(input.clone()).or_default() += 1;
            if let Some(value) = self.data.get(input) {
                Ok(value.clone())
            } else {
                Err(Error::NotFound)
            }
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
        let resolver = MaybeCachedResolver::new(mock_resolver(counts.clone()), None);
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
                Some(value) => assert_eq!(result.expect("failed to resolve"), value),
                None => assert!(result.is_err()),
            }
        }
        assert_eq!(
            *counts.read().await,
            [
                (String::from("k1"), 3),
                (String::from("k2"), 2),
                (String::from("k3"), 2),
            ]
            .into_iter()
            .collect()
        );
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    async fn test_cached() {
        let counts = Arc::new(RwLock::new(HashMap::new()));
        let resolver =
            MaybeCachedResolver::new(mock_resolver(counts.clone()), Some(Default::default()));
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
                Some(value) => assert_eq!(result.expect("failed to resolve"), value),
                None => assert!(result.is_err()),
            }
        }
        assert_eq!(
            *counts.read().await,
            [
                (String::from("k1"), 1),
                (String::from("k2"), 1),
                (String::from("k3"), 2),
            ]
            .into_iter()
            .collect()
        );
    }

    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_cached_with_max_capacity() {
        let counts = Arc::new(RwLock::new(HashMap::new()));
        let resolver = MaybeCachedResolver::new(
            mock_resolver(counts.clone()),
            Some(CachedResolverConfig {
                max_capacity: Some(1),
                ..Default::default()
            }),
        );
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
                Some(value) => assert_eq!(result.expect("failed to resolve"), value),
                None => assert!(result.is_err()),
            }
        }
        assert_eq!(
            *counts.read().await,
            [
                (String::from("k1"), 2),
                (String::from("k2"), 1),
                (String::from("k3"), 2),
            ]
            .into_iter()
            .collect()
        );
    }

    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_cached_with_time_to_live() {
        let counts = Arc::new(RwLock::new(HashMap::new()));
        let resolver = MaybeCachedResolver::new(
            mock_resolver(counts.clone()),
            Some(CachedResolverConfig {
                time_to_live: Some(Duration::from_millis(10)),
                ..Default::default()
            }),
        );
        for _ in 0..10 {
            let result = resolver.resolve(&String::from("k1")).await;
            assert_eq!(result.expect("failed to resolve"), "v1");
        }
        sleep(Duration::from_millis(10)).await;
        for _ in 0..10 {
            let result = resolver.resolve(&String::from("k1")).await;
            assert_eq!(result.expect("failed to resolve"), "v1");
        }
        assert_eq!(
            *counts.read().await,
            [(String::from("k1"), 2)].into_iter().collect()
        );
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    async fn test_throttled() {
        let counts = Arc::new(RwLock::new(HashMap::new()));
        let resolver = Arc::new(ThrottledResolver::new(mock_resolver(counts.clone())));

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
                Some(value) => assert_eq!(result.expect("failed to resolve"), value),
                None => assert!(result.is_err()),
            }
        }
        assert_eq!(
            *counts.read().await,
            [
                (String::from("k1"), 1),
                (String::from("k2"), 1),
                (String::from("k3"), 1),
            ]
            .into_iter()
            .collect()
        );
    }
}
