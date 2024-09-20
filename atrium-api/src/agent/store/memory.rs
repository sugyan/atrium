use super::{Session, SessionStore};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default, Clone)]
pub struct MemorySessionStore {
    session: Arc<RwLock<Option<Session>>>,
}

impl SessionStore for MemorySessionStore {
    async fn get_session(&self) -> Option<Session> {
        self.session.read().await.clone()
    }
    async fn set_session(&self, session: Session) {
        self.session.write().await.replace(session);
    }
    async fn clear_session(&self) {
        self.session.write().await.take();
    }
}
