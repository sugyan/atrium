use crate::{
    store::session::{Session, SessionStore},
    TokenSet,
};
use atrium_api::types::string::Did;
use std::sync::Arc;
use thiserror::Error;

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

        self.store.set(self.sub.clone(), self.session.clone()).await.ok();
        // Might this be done asynchronously?
        // let store = Arc::clone(&self.store);
        // let sub = self.sub.clone();
        // let session = self.session.clone();
        // tokio::spawn(async move {
        //     store.set(sub, session).await.ok();
        // });
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("session store error: {0}")]
    Store(Box<dyn std::error::Error>),
    #[error("session does not exist")]
    SessionNotFound,
}

#[derive(Debug)]
pub struct SessionRegistry<S> {
    store: Arc<S>,
}

impl<S> SessionRegistry<S> {
    pub fn new(store: S) -> Self {
        Self { store: Arc::new(store) }
    }
}

impl<S> SessionRegistry<S>
where
    S: SessionStore + Send + Sync + 'static,
{
    pub async fn get(&self, key: &Did) -> Result<SessionHandle<S>, Error> {
        let session = self
            .store
            .get(key)
            .await
            .map_err(|e| Error::Store(Box::new(e)))?
            .ok_or(Error::SessionNotFound)?;
        Ok(SessionHandle::new(session, Arc::clone(&self.store), key.clone()))
    }
    pub async fn set(&self, key: Did, value: Session) -> Result<SessionHandle<S>, S::Error> {
        self.store.set(key.clone(), value.clone()).await?;
        Ok(SessionHandle::new(value, Arc::clone(&self.store), key))
    }
    pub async fn del(&self, key: &Did) -> Result<(), S::Error> {
        self.store.del(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OAuthTokenType;
    use atrium_api::types::string::Datetime;
    use atrium_common::store::Store;
    use std::{collections::HashMap, ops::DerefMut, time::Duration};
    use thiserror::Error;
    use tokio::{
        sync::{Mutex, RwLock},
        time::sleep,
    };

    #[derive(Error, Debug)]
    enum MockStoreError {}

    struct MockSessionStore {
        hm: Mutex<HashMap<Did, Session>>,
    }

    impl Store<Did, Session> for MockSessionStore {
        type Error = MockStoreError;

        async fn get(&self, key: &Did) -> Result<Option<Session>, Self::Error> {
            sleep(Duration::from_micros(10)).await;
            Ok(self.hm.lock().await.get(key).cloned())
        }
        async fn set(&self, key: Did, value: Session) -> Result<(), Self::Error> {
            sleep(Duration::from_micros(10)).await;
            self.hm.lock().await.insert(key, value);
            Ok(())
        }
        async fn del(&self, key: &Did) -> Result<(), Self::Error> {
            sleep(Duration::from_micros(10)).await;
            self.hm.lock().await.remove(key);
            Ok(())
        }
        async fn clear(&self) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    impl SessionStore for MockSessionStore {}

    impl Default for MockSessionStore {
        fn default() -> Self {
            Self { hm: Mutex::new(HashMap::from_iter([(did(), session())])) }
        }
    }

    fn did() -> Did {
        "did:fake:handle.test".parse().expect("invalid did")
    }

    fn session() -> Session {
        let dpop_key = serde_json::from_str(
            r#"{
                "kty": "EC",
                "crv": "P-256",
                "x": "NIRNgPVAwnVNzN5g2Ik2IMghWcjnBOGo9B-lKXSSXFs",
                "y": "iWF-Of43XoSTZxcadO9KWdPTjiCoviSztYw7aMtZZMc",
                "d": "9MuCYfKK4hf95p_VRj6cxKJwORTgvEU3vynfmSgFH2M"
            }"#,
        )
        .expect("key should be valid");
        let token_set = TokenSet {
            iss: String::from("https://iss.example.com"),
            sub: "did:fake:sub.test".parse().expect("invalid did"),
            aud: String::from("https://aud.example.com"),
            scope: None,
            refresh_token: Some(String::from("refreshtoken")),
            access_token: String::from("accesstoken"),
            token_type: OAuthTokenType::DPoP,
            expires_at: None,
        };
        Session { dpop_key, token_set }
    }

    #[tokio::test]
    async fn test_get_handle() -> Result<(), Box<dyn std::error::Error>> {
        let registry = SessionRegistry::new(MockSessionStore::default());
        let result = registry.get(&"did:fake:nonexistent".parse()?).await;
        assert!(matches!(result, Err(Error::SessionNotFound)));
        let result = registry.get(&"did:fake:handle.test".parse()?).await;
        let handle = result.expect("handle should exist");
        assert_eq!(handle.read().await.token_set.access_token, "accesstoken");
        Ok(())
    }

    #[tokio::test]
    async fn test_handle_update() -> Result<(), Box<dyn std::error::Error>> {
        let store = MockSessionStore::default();
        let registry = SessionRegistry::new(store);
        let mut handle = registry.get(&did()).await?;
        assert_eq!(handle.read().await.token_set.access_token, "accesstoken");
        // update token set
        handle
            .write_token_set(TokenSet {
                iss: String::from("https://iss.example.com"),
                sub: "did:fake:sub.test".parse().expect("invalid did"),
                aud: String::from("https://aud.example.com"),
                scope: None,
                refresh_token: Some(String::from("refreshtoken")),
                access_token: String::from("newaccesstoken"),
                token_type: OAuthTokenType::DPoP,
                expires_at: None,
            })
            .await;
        // check if the token set is updated
        assert_eq!(handle.read().await.token_set.access_token, "newaccesstoken");
        match registry.store.get(&did()).await? {
            Some(session) => {
                assert_eq!(session.token_set.access_token, "newaccesstoken");
            }
            None => {
                panic!("session should exist");
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_parallel() -> Result<(), Box<dyn std::error::Error>> {
        async fn update_with_lock(
            registry: Arc<SessionRegistry<MockSessionStore>>,
        ) -> Result<(bool, String), String> {
            let session =
                Arc::new(RwLock::new(registry.get(&did()).await.map_err(|e| e.to_string())?));

            let mut handle = session.write().await;
            let mut token_set = handle.read().await.token_set;
            if token_set.expires_at.is_some() {
                return Ok((false, handle.read().await.token_set.access_token));
            }
            token_set.access_token = String::from("newaccesstoken");
            token_set.expires_at = Some(Datetime::now());
            handle.deref_mut().write_token_set(token_set).await;

            Ok((true, handle.read().await.token_set.access_token))
        }

        let store = MockSessionStore::default();
        let registry = Arc::new(SessionRegistry::new(store));
        let mut handles = Vec::new();
        for _ in 1..5 {
            let registry = Arc::clone(&registry);
            handles.push(tokio::spawn(async { update_with_lock(registry).await }));
        }
        let mut refreshed_count = 0;
        for handle in handles {
            let (refreshed, access_token) = handle.await??;
            assert_eq!(access_token, "newaccesstoken");
            if refreshed {
                refreshed_count += 1;
            }
        }
        assert_eq!(refreshed_count, 1);
        Ok(())
    }
}
