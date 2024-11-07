use super::{AtpSession, AtpSessionStore};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default, Clone)]
pub struct MemorySessionStore {
    session: Arc<RwLock<Option<AtpSession>>>,
}

impl AtpSessionStore for MemorySessionStore {
    async fn get_session(&self) -> Option<AtpSession> {
        self.session.read().await.clone()
    }
    async fn set_session(&self, session: AtpSession) {
        self.session.write().await.replace(session);
    }
    async fn clear_session(&self) {
        self.session.write().await.take();
    }
}
