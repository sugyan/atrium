use super::AtpSession;
use atrium_common::store::{memory::MemoryStore, Store};
use std::future::Future;

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait AtpSessionStore: Store<(), AtpSession>
where
    Self: Send + Sync,
{
    fn get_session(&self) -> impl Future<Output = Option<AtpSession>> {
        async { self.get(&()).await.ok().flatten() }
    }
    fn set_session(&self, session: AtpSession) -> impl Future<Output = ()> {
        async {
            self.set((), session).await.ok();
        }
    }
    fn clear_session(&self) -> impl Future<Output = ()> {
        async {
            self.clear().await.ok();
        }
    }
}

pub type MemorySessionStore = MemoryStore<(), AtpSession>;

impl AtpSessionStore for MemorySessionStore {}
