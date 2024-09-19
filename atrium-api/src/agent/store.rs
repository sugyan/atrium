mod memory;

use std::future::Future;

pub use self::memory::MemorySessionStore;
pub(crate) use super::Session;

pub trait SessionStore {
    #[must_use]
    fn get_session(&self) -> impl Future<Output = Option<Session>> + Send;
    #[must_use]
    fn set_session(&self, session: Session) -> impl Future<Output = ()> + Send;
    #[must_use]
    fn clear_session(&self) -> impl Future<Output = ()> + Send;
}
