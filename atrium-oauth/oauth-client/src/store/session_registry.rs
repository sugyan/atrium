use crate::{
    server_agent::OAuthServerFactory,
    store::session::{Session, SessionStore},
};
use atrium_api::{
    did_doc::DidDocument,
    types::string::{Datetime, Did},
};
use atrium_common::resolver::Resolver;
use atrium_identity::handle::HandleResolver;
use atrium_xrpc::HttpClient;
use dashmap::DashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ServerAgent(#[from] crate::server_agent::Error),
    #[error("session store error: {0}")]
    Store(String),
    #[error("session does not exist")]
    SessionNotFound,
}

pub struct SessionRegistry<S, T, D, H>
where
    S: SessionStore + Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
{
    store: Arc<S>,
    server_factory: Arc<OAuthServerFactory<T, D, H>>,
    pending: DashMap<Did, Arc<Mutex<()>>>,
}

impl<S, T, D, H> SessionRegistry<S, T, D, H>
where
    S: SessionStore + Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
{
    pub fn new(store: S, server_factory: Arc<OAuthServerFactory<T, D, H>>) -> Self {
        let store = Arc::new(store);
        Self { store: Arc::clone(&store), server_factory, pending: DashMap::new() }
    }
}

impl<S, T, D, H> SessionRegistry<S, T, D, H>
where
    S: SessionStore + Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
    D: Resolver<Input = Did, Output = DidDocument, Error = atrium_identity::Error> + Send + Sync,
    H: HandleResolver + Send + Sync + 'static,
{
    async fn get_refreshed(&self, key: &Did) -> Result<Session, Error> {
        let lock =
            self.pending.entry(key.clone()).or_insert_with(|| Arc::new(Mutex::new(()))).clone();
        let _guard = lock.lock().await;

        let mut session = self
            .store
            .get(key)
            .await
            .map_err(|e| Error::Store(e.to_string()))?
            .ok_or(Error::SessionNotFound)?;
        if let Some(expires_at) = &session.token_set.expires_at {
            if expires_at > &Datetime::now() {
                return Ok(session);
            }
        }

        let server = self
            .server_factory
            .build_from_issuer(session.dpop_key.clone(), &session.token_set.iss)
            .await?;
        session.token_set = server.refresh(&session.token_set).await?;
        self.store
            .set(key.clone(), session.clone())
            .await
            .map_err(|e| Error::Store(e.to_string()))?;
        Ok(session)
    }
    pub async fn get(&self, key: &Did, refresh: bool) -> Result<Session, Error> {
        if refresh {
            self.get_refreshed(key).await
        } else {
            // TODO: cached?
            self.store
                .get(key)
                .await
                .map_err(|e| Error::Store(e.to_string()))?
                .ok_or(Error::SessionNotFound)
        }
    }
    pub async fn set(&self, key: Did, value: Session) -> Result<(), S::Error> {
        self.store.set(key.clone(), value.clone()).await
    }
    pub async fn del(&self, key: &Did) -> Result<(), S::Error> {
        self.store.del(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        tests::{
            client_metadata, dpop_key, oauth_resolver, protected_resource_metadata,
            server_metadata, MockDidResolver, NoopHandleResolver,
        },
        types::{OAuthTokenResponse, OAuthTokenType, RefreshRequestParameters, TokenSet},
    };
    use atrium_common::store::Store;
    use atrium_xrpc::http::{header::CONTENT_TYPE, Request, Response, StatusCode};
    use std::{collections::HashMap, time::Duration};
    use tokio::{sync::Mutex, time::sleep};

    #[derive(Error, Debug)]
    enum MockStoreError {}

    struct MockHttpClient {
        next_token: Arc<Mutex<Option<OAuthTokenResponse>>>,
    }

    impl Default for MockHttpClient {
        fn default() -> Self {
            Self {
                next_token: Arc::new(Mutex::new(Some(OAuthTokenResponse {
                    access_token: String::from("new_accesstoken"),
                    token_type: OAuthTokenType::DPoP,
                    expires_in: Some(10),
                    refresh_token: Some(String::from("new_refreshtoken")),
                    scope: None,
                    sub: None,
                }))),
            }
        }
    }

    impl HttpClient for MockHttpClient {
        async fn send_http(
            &self,
            request: Request<Vec<u8>>,
        ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
            println!("{:?}", request);

            Ok(match (request.uri().host(), request.uri().path()) {
                (Some("iss.example.com"), "/.well-known/oauth-authorization-server") => {
                    Response::builder()
                        .header(CONTENT_TYPE, "application/json")
                        .body(serde_json::to_vec(&server_metadata())?)?
                }
                (Some("aud.example.com"), "/.well-known/oauth-protected-resource") => {
                    Response::builder()
                        .header(CONTENT_TYPE, "application/json")
                        .body(serde_json::to_vec(&protected_resource_metadata())?)?
                }
                (Some("iss.example.com"), "/token") => {
                    let parameters =
                        serde_html_form::from_bytes::<RefreshRequestParameters>(request.body())?;
                    if let Some(token_response) = if parameters.refresh_token == "refreshtoken" {
                        self.next_token.lock().await.take()
                    } else {
                        None
                    } {
                        Response::builder()
                            .header(CONTENT_TYPE, "application/json")
                            .body(serde_json::to_vec(&token_response)?)?
                    } else {
                        Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .header("WWW-Authenticate", "DPoP error=\"invalid_token\"")
                            .body(Vec::new())?
                    }
                }
                _ => {
                    Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Vec::new())?
                }
            })
        }
    }

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
        let dpop_key = dpop_key();
        let token_set = TokenSet {
            iss: String::from("https://iss.example.com"),
            sub: did(),
            aud: String::from("https://aud.example.com"),
            scope: None,
            refresh_token: Some(String::from("refreshtoken")),
            access_token: String::from("accesstoken"),
            token_type: OAuthTokenType::DPoP,
            expires_at: None,
        };
        Session { dpop_key, token_set }
    }

    fn session_registry(
        store: MockSessionStore,
    ) -> SessionRegistry<MockSessionStore, MockHttpClient, MockDidResolver, NoopHandleResolver>
    {
        let http_client = Arc::new(MockHttpClient::default());
        SessionRegistry::new(
            store,
            Arc::new(OAuthServerFactory::new(
                client_metadata(),
                Arc::new(oauth_resolver(Arc::clone(&http_client))),
                http_client,
                None,
            )),
        )
    }

    #[tokio::test]
    async fn test_get_session() -> Result<(), Box<dyn std::error::Error>> {
        let registry = session_registry(MockSessionStore::default());
        let result = registry.get(&"did:fake:nonexistent".parse()?, false).await;
        assert!(matches!(result, Err(Error::SessionNotFound)));
        let result = registry.get(&"did:fake:handle.test".parse()?, false).await;
        let session = result.expect("handle should exist");
        assert_eq!(session.token_set.access_token, "accesstoken");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_refreshed() -> Result<(), Box<dyn std::error::Error>> {
        let registry = session_registry(MockSessionStore::default());
        let session = registry.get(&did(), true).await?;
        assert_eq!(session.token_set.access_token, "new_accesstoken");
        assert_eq!(session.token_set.refresh_token.as_deref(), Some("new_refreshtoken"));
        // second time should return the same session
        let session = registry.get(&did(), true).await?;
        assert_eq!(session.token_set.access_token, "new_accesstoken");
        assert_eq!(session.token_set.refresh_token.as_deref(), Some("new_refreshtoken"));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_refreshed_parallel() -> Result<(), Box<dyn std::error::Error>> {
        let registry = Arc::new(session_registry(MockSessionStore::default()));
        let mut handles = Vec::new();
        for _ in 0..3 {
            let registry = Arc::clone(&registry);
            handles.push(tokio::spawn(async move { registry.get(&did(), true).await }));
        }
        for result in futures::future::join_all(handles).await {
            match result? {
                Ok(session) => {
                    assert_eq!(session.token_set.access_token, "new_accesstoken");
                    assert_eq!(
                        session.token_set.refresh_token.as_deref(),
                        Some("new_refreshtoken")
                    );
                }
                Err(err) => {
                    panic!("unexpected error: {err:?}");
                }
            }
        }
        Ok(())
    }
}
