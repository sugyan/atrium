use super::store::OAuthSessionStore;
use crate::{server_agent::OAuthServerAgent, store::session::Session, DpopClient};
use atrium_api::{
    agent::{CloneWithProxy, Configure, InnerStore, WrapperClient},
    types::string::Did,
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
    inner: WrapperClient<S, DpopClient<T>, String>,
    store: Arc<InnerStore<S, String>>,
    server_agent: OAuthServerAgent<T, D, H>,
    session: Arc<RwLock<Session>>,
}

impl<S, T, D, H> Client<S, T, D, H>
where
    T: HttpClient + Send + Sync + 'static,
{
    pub fn new(
        store: Arc<InnerStore<S, String>>,
        xrpc: DpopClient<T>,
        server_agent: OAuthServerAgent<T, D, H>,
        session: Arc<RwLock<Session>>,
    ) -> Self {
        let inner = WrapperClient::new(Arc::clone(&store), xrpc);
        Self { inner, store, server_agent, session }
    }
}

impl<S, T, D, H> Client<S, T, D, H>
where
    S: OAuthSessionStore + Send + Sync,
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
                .map_or(false, |s| s.starts_with("DPoP ") && s.contains("error=\"invalid_token\"")),
            _ => false,
        }
    }
    async fn refresh_token(&self) {
        let token_set = self.session.read().await.token_set.clone();
        if let Ok(refreshed) = self.server_agent.refresh(&token_set).await {
            self.store.set(refreshed.access_token.clone()).await;
            self.session.write().await.token_set = refreshed;
        }
    }
}

impl<S, T, D, H> HttpClient for Client<S, T, D, H>
where
    S: OAuthSessionStore + Send + Sync,
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
    S: OAuthSessionStore + Send + Sync,
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
    WrapperClient<S, T, String>: CloneWithProxy,
{
    fn clone_with_proxy(&self, did: Did, service_type: impl AsRef<str>) -> Self {
        Self {
            inner: self.inner.clone_with_proxy(did, service_type),
            store: Arc::clone(&self.store),
            server_agent: self.server_agent.clone(),
            session: Arc::clone(&self.session),
        }
    }
}
