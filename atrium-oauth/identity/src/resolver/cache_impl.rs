#[cfg(not(target_arch = "wasm32"))]
mod moka;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use self::moka::MokaCache as CacheImpl;
#[cfg(target_arch = "wasm32")]
pub use self::wasm::WasmCache as CacheImpl;
