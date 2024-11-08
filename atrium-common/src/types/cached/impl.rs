#[cfg(not(target_arch = "wasm32"))]
mod moka;
#[cfg(target_arch = "wasm32")]
mod wasm;

use std::future::Future;
use std::hash::Hash;

#[cfg(not(target_arch = "wasm32"))]
pub use self::moka::MokaCache as CacheImpl;
#[cfg(target_arch = "wasm32")]
pub use self::wasm::WasmCache as CacheImpl;

use super::CacheConfig;

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub(crate) trait Cache {
    type Input: Hash + Eq + Sync + 'static;
    type Output: Clone + Sync + 'static;

    fn new(config: CacheConfig) -> Self;
    fn get(&self, key: &Self::Input) -> impl Future<Output = Option<Self::Output>>;
    fn set(&self, key: Self::Input, value: Self::Output) -> impl Future<Output = ()>;
}
