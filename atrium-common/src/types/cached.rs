pub mod r#impl;

use std::fmt::Debug;
use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub struct CacheConfig {
    pub max_capacity: Option<u64>,
    pub time_to_live: Option<Duration>,
}

pub trait Cacheable<C>
where
    Self: Sized,
{
    fn cached(self, cache: C) -> Cached<Self, C>;
}

impl<T, C> Cacheable<C> for T {
    fn cached(self, cache: C) -> Cached<Self, C> {
        Cached::new(self, cache)
    }
}

pub struct Cached<T, C> {
    pub inner: T,
    pub cache: C,
}

impl<T, C> Cached<T, C> {
    pub fn new(inner: T, cache: C) -> Self {
        Self { inner, cache }
    }
}
