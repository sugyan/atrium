mod memory;

use std::future::Future;

pub use self::memory::MemorySessionStore;
pub(crate) use super::Session;

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait SessionStore {
    #[must_use]
    fn get_session(&self) -> impl Future<Output = Option<Session>>;
    #[must_use]
    fn set_session(&self, session: Session) -> impl Future<Output = ()>;
    #[must_use]
    fn clear_session(&self) -> impl Future<Output = ()>;
}
