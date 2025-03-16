use super::store::MemorySessionStore;
use crate::{
    server_agent::OAuthServerAgent,
    store::{session::SessionStore, session_registry::SessionHandle},
    DpopClient,
};
use atrium_api::{
    agent::{
        utils::{SessionClient, SessionWithEndpointStore},
        CloneWithProxy, Configure,
    },
    types::string::{Datetime, Did},
};
use atrium_identity::{did::DidResolver, handle::HandleResolver};
use atrium_xrpc::{
    http::{Request, Response},
    Error, HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, sync::Arc};
use tokio::sync::RwLock;

pub struct Client<S, T, D, H>
where
    T: HttpClient + Send + Sync + 'static,
{
    inner: SessionClient<MemorySessionStore, DpopClient<T>, String>,
    store: Arc<SessionWithEndpointStore<MemorySessionStore, String>>,
    server_agent: OAuthServerAgent<T, D, H>,
    session: Arc<RwLock<SessionHandle<S>>>,
}

impl<S, T, D, H> Client<S, T, D, H>
where
    T: HttpClient + Send + Sync + 'static,
{
    pub fn new(
        store: Arc<SessionWithEndpointStore<MemorySessionStore, String>>,
        xrpc: DpopClient<T>,
        server_agent: OAuthServerAgent<T, D, H>,
        session: Arc<RwLock<SessionHandle<S>>>,
    ) -> Self {
        let inner = SessionClient::new(Arc::clone(&store), xrpc);
        Self { inner, store, server_agent, session }
    }
}

impl<S, T, D, H> Client<S, T, D, H>
where
    S: SessionStore + Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
{
    // https://datatracker.ietf.org/doc/html/rfc6750#section-3
    fn is_invalid_token_response<O, E>(result: &Result<OutputDataOrBytes<O>, Error<E>>) -> bool
    where
        O: DeserializeOwned + Send + Sync,
        E: DeserializeOwned + Send + Sync + Debug,
    {
        match result {
            Err(Error::Authentication(value)) => value
                .to_str()
                .is_ok_and(|s| s.starts_with("DPoP ") && s.contains("error=\"invalid_token\"")),
            _ => false,
        }
    }
    async fn refresh_token(&self) {
        let mut handle = self.session.write().await;
        let token_set = handle.session().token_set;
        if let Some(expired_at) = &token_set.expires_at {
            if *expired_at > Datetime::now() {
                return;
            }
        }
        if let Ok(refreshed) = self.server_agent.refresh(&token_set).await {
            let _ = self.store.set(refreshed.access_token.clone()).await;
            handle.write_token_set(refreshed).await;
        } else {
            let _ = self.store.clear().await;
        }
    }
}

impl<S, T, D, H> HttpClient for Client<S, T, D, H>
where
    S: Send + Sync,
    T: HttpClient + Send + Sync + 'static,
    D: Send + Sync,
    H: Send + Sync,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.inner.send_http(request).await
    }
}

impl<S, T, D, H> XrpcClient for Client<S, T, D, H>
where
    S: SessionStore + Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
{
    fn base_uri(&self) -> String {
        self.inner.base_uri()
    }
    async fn send_xrpc<P, I, O, E>(
        &self,
        request: &XrpcRequest<P, I>,
    ) -> Result<OutputDataOrBytes<O>, Error<E>>
    where
        P: Serialize + Send + Sync,
        I: Serialize + Send + Sync,
        O: DeserializeOwned + Send + Sync,
        E: DeserializeOwned + Send + Sync + Debug,
    {
        let result = self.inner.send_xrpc(request).await;
        // handle session-refreshes as needed
        if Self::is_invalid_token_response(&result) {
            self.refresh_token().await;
            self.inner.send_xrpc(request).await
        } else {
            result
        }
    }
}

impl<S, T, D, H> Configure for Client<S, T, D, H>
where
    T: HttpClient + Send + Sync + 'static,
{
    fn configure_endpoint(&self, endpoint: String) {
        self.inner.configure_endpoint(endpoint)
    }
    /// Configures the moderation services to be applied on requests.
    fn configure_labelers_header(&self, labeler_dids: Option<Vec<(Did, bool)>>) {
        self.inner.configure_labelers_header(labeler_dids)
    }
    /// Configures the atproto-proxy header to be applied on requests.
    fn configure_proxy_header(&self, did: Did, service_type: impl AsRef<str>) {
        self.inner.configure_proxy_header(did, service_type)
    }
}

impl<S, T, D, H> CloneWithProxy for Client<S, T, D, H>
where
    T: HttpClient + Send + Sync + 'static,
    SessionClient<S, T, String>: CloneWithProxy,
{
    fn clone_with_proxy(&self, did: Did, service_type: impl AsRef<str>) -> Self {
        Self {
            inner: self.inner.clone_with_proxy(did, service_type),
            store: Arc::clone(&self.store),
            server_agent: self.server_agent.clone(),
            session: self.session.clone(),
        }
    }
}
