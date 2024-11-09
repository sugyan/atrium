use std::future::Future;

use atrium_common::store::{memory::MemoryCellStore, CellStore};

pub(crate) use super::AtpSession;

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait AtpSessionStore: CellStore<AtpSession> {
    fn get_session(&self) -> impl Future<Output = Option<AtpSession>>;
    fn set_session(&self, session: AtpSession) -> impl Future<Output = ()>;
    fn clear_session(&self) -> impl Future<Output = ()>;
}

impl<T> AtpSessionStore for T
where
    T: CellStore<AtpSession> + Send + Sync,
{
    async fn get_session(&self) -> Option<AtpSession> {
        self.get().await.expect("Infallible")
    }
    async fn set_session(&self, session: AtpSession) {
        self.set(session).await.expect("Infallible")
    }
    async fn clear_session(&self) {
        self.clear().await.expect("Infallible")
    }
}

pub type MemorySessionStore = MemoryCellStore<AtpSession>;
