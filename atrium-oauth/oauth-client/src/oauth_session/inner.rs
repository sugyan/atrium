use super::store::MemorySessionStore;
use crate::{
    store::{session::SessionStore, session_registry::SessionRegistry},
    DpopClient,
};
use atrium_api::{
    agent::{
        utils::{SessionClient, SessionWithEndpointStore},
        CloneWithProxy, Configure,
    },
    did_doc::DidDocument,
    types::string::{Did, Handle},
};
use atrium_common::resolver::Resolver;
use atrium_xrpc::{
    http::{Request, Response},
    Error, HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, sync::Arc};

pub struct Client<S, T, D, H>
where
    S: SessionStore + Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
{
    inner: SessionClient<MemorySessionStore, DpopClient<T>, String>,
    store: Arc<SessionWithEndpointStore<MemorySessionStore, String>>,
    sub: Did,
    session_registry: Arc<SessionRegistry<S, T, D, H>>,
}

impl<S, T, D, H> Client<S, T, D, H>
where
    S: SessionStore + Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
{
    pub fn new(
        store: Arc<SessionWithEndpointStore<MemorySessionStore, String>>,
        xrpc: DpopClient<T>,
        sub: Did,
        session_registry: Arc<SessionRegistry<S, T, D, H>>,
    ) -> Self {
        let inner = SessionClient::new(Arc::clone(&store), xrpc);
        Self { inner, store, sub, session_registry }
    }
}

impl<S, T, D, H> Client<S, T, D, H>
where
    S: SessionStore + Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
    D: Resolver<Input = Did, Output = DidDocument, Error = atrium_identity::Error> + Send + Sync,
    H: Resolver<Input = Handle, Output = Did, Error = atrium_identity::Error> + Send + Sync,
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
        if let Ok(session) = self.session_registry.get(&self.sub, true).await {
            let _ = self.store.set(session.token_set.access_token.clone()).await;
        }
    }
}

impl<S, T, D, H> HttpClient for Client<S, T, D, H>
where
    S: SessionStore + Send + Sync + 'static,
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
    D: Resolver<Input = Did, Output = DidDocument, Error = atrium_identity::Error> + Send + Sync,
    H: Resolver<Input = Handle, Output = Did, Error = atrium_identity::Error> + Send + Sync,
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
    S: SessionStore + Send + Sync + 'static,
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
    S: SessionStore + Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
    SessionClient<S, T, String>: CloneWithProxy,
{
    fn clone_with_proxy(&self, did: Did, service_type: impl AsRef<str>) -> Self {
        Self {
            inner: self.inner.clone_with_proxy(did, service_type),
            store: Arc::clone(&self.store),
            sub: self.sub.clone(),
            session_registry: Arc::clone(&self.session_registry),
        }
    }
}
