use crate::{
    store::session::{Session, SessionStore},
    TokenSet,
};
use atrium_api::types::string::Did;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SessionHandle<S> {
    session: Session,
    store: Arc<S>,
    sub: Did,
}

impl<S> SessionHandle<S>
where
    S: SessionStore + Send + Sync + 'static,
{
    pub(crate) fn new(session: Session, store: Arc<S>, sub: Did) -> Self {
        Self { session, store, sub }
    }
    pub async fn read(&self) -> Session {
        self.session.clone()
    }
    pub async fn write_token_set(&mut self, value: TokenSet) {
        self.session.token_set = value;
        // write to store asynchronously
        let store = Arc::clone(&self.store);
        let sub = self.sub.clone();
        let session = self.session.clone();
        tokio::spawn(async move {
            store.set(sub, session).await.ok();
        });
    }
}

#[derive(Debug)]
pub struct SessionGetter<S> {
    store: Arc<S>,
}

impl<S> SessionGetter<S> {
    pub fn new(store: S) -> Self {
        Self { store: Arc::new(store) }
    }
}

impl<S> SessionGetter<S>
where
    S: SessionStore + Send + Sync + 'static,
{
    pub async fn get(&self, key: &Did) -> Result<Option<SessionHandle<S>>, S::Error> {
        self.store
            .get(key)
            .await?
            .map(|session| Ok(SessionHandle::new(session, Arc::clone(&self.store), key.clone())))
            .transpose()
    }
    pub async fn set(&self, key: Did, value: Session) -> Result<(), S::Error> {
        self.store.set(key, value).await
    }
}
