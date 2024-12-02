use crate::store::session::{Session, SessionStore};
use atrium_api::types::string::Did;
use atrium_common::store::Store;
use std::sync::Arc;

#[derive(Debug)]
pub struct SessionGetter<S> {
    store: Arc<S>,
}

impl<S> SessionGetter<S> {
    pub fn new(store: S) -> Self {
        Self { store: Arc::new(store) }
    }
    // TODO: extended store methods?
}

impl<S> Clone for SessionGetter<S> {
    fn clone(&self) -> Self {
        Self { store: self.store.clone() }
    }
}

impl<S> Store<Did, Session> for SessionGetter<S>
where
    S: SessionStore + Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    type Error = S::Error;
    async fn get(&self, key: &Did) -> Result<Option<Session>, Self::Error> {
        self.store.get(key).await
    }
    async fn set(&self, key: Did, value: Session) -> Result<(), Self::Error> {
        self.store.set(key, value).await
    }
    async fn del(&self, key: &Did) -> Result<(), Self::Error> {
        self.store.del(key).await
    }
    async fn clear(&self) -> Result<(), Self::Error> {
        self.store.clear().await
    }
}

impl<S> SessionStore for SessionGetter<S>
where
    S: SessionStore + Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
}
