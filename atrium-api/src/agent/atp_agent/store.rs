mod memory;

use std::future::Future;

pub use self::memory::MemorySessionStore;
pub(crate) use super::AtpSession;

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait AtpSessionStore {
    #[must_use]
    fn get_session(&self) -> impl Future<Output = Option<AtpSession>>;
    #[must_use]
    fn set_session(&self, session: AtpSession) -> impl Future<Output = ()>;
    #[must_use]
    fn clear_session(&self) -> impl Future<Output = ()>;
}
