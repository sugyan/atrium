mod memory;

pub use self::memory::MemorySessionStore;
pub(crate) use super::Session;
use async_trait::async_trait;

#[async_trait]
pub trait SessionStore {
    #[must_use]
    async fn get_session(&self) -> Option<Session>;
    #[must_use]
    async fn set_session(&self, session: Session);
    #[must_use]
    async fn clear_session(&self);
}
