use crate::did_doc::DidDocument;
use crate::types::string::Did;
use crate::types::TryFromUnknown;
use atrium_common::store::Store as StoreTrait;
use atrium_xrpc::error::{Error, Result, XrpcErrorKind};
use atrium_xrpc::types::AuthorizationToken;
use atrium_xrpc::{HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest};
use http::{Method, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::hash::Hash;
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};
use tokio::sync::{Mutex, Notify};

use super::AtpSession;

struct WrapperClient<S, T> {
    store: Arc<Store<S>>,
    proxy_header: RwLock<Option<String>>,
    labelers_header: Arc<RwLock<Option<Vec<String>>>>,
    inner: Arc<T>,
}

impl<S, T> WrapperClient<S, T> {
    fn configure_proxy_header(&self, value: String) {
        self.proxy_header.write().expect("failed to write proxy header").replace(value);
    }
    fn configure_labelers_header(&self, labelers_dids: Option<Vec<(Did, bool)>>) {
        *self.labelers_header.write().expect("failed to write labelers header") =
            labelers_dids.map(|dids| {
                dids.iter()
                    .map(|(did, redact)| {
                        if *redact {
                            format!("{};redact", did.as_ref())
                        } else {
                            did.as_ref().into()
                        }
                    })
                    .collect()
            })
    }
}

impl<S, T> Clone for WrapperClient<S, T> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            labelers_header: self.labelers_header.clone(),
            proxy_header: RwLock::new(
                self.proxy_header.read().expect("failed to read proxy header").clone(),
            ),
            inner: self.inner.clone(),
        }
    }
}

impl<S, T> HttpClient for WrapperClient<S, T>
where
    S: Send + Sync,
    T: HttpClient + Send + Sync,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> core::result::Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>>
    {
        self.inner.send_http(request).await
    }
}

impl<S, T> XrpcClient for WrapperClient<S, T>
where
    S: StoreTrait<(), AtpSession> + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    fn base_uri(&self) -> String {
        self.store.get_endpoint()
    }
    async fn authorization_token(&self, is_refresh: bool) -> Option<AuthorizationToken> {
        self.store.get(&()).await.transpose().and_then(core::result::Result::ok).map(|session| {
            AuthorizationToken::Bearer(if is_refresh {
                session.data.refresh_jwt
            } else {
                session.data.access_jwt
            })
        })
    }
    async fn atproto_proxy_header(&self) -> Option<String> {
        self.proxy_header.read().expect("failed to read proxy header").clone()
    }
    async fn atproto_accept_labelers_header(&self) -> Option<Vec<String>> {
        self.labelers_header.read().expect("failed to read labelers header").clone()
    }
}

pub struct Client<S, T> {
    store: Arc<Store<S>>,
    inner: WrapperClient<S, T>,
    is_refreshing: Arc<Mutex<bool>>,
    notify: Arc<Notify>,
}

impl<S, T> Client<S, T>
where
    S: StoreTrait<(), AtpSession> + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    pub fn new(store: Arc<Store<S>>, xrpc: T) -> Self {
        let inner = WrapperClient {
            store: Arc::clone(&store),
            labelers_header: Arc::new(RwLock::new(None)),
            proxy_header: RwLock::new(None),
            inner: Arc::new(xrpc),
        };
        Self {
            store,
            inner,
            is_refreshing: Arc::new(Mutex::new(false)),
            notify: Arc::new(Notify::new()),
        }
    }
    pub fn configure_endpoint(&self, endpoint: String) {
        *self.store.endpoint.write().expect("failed to write endpoint") = endpoint;
    }
    pub fn configure_proxy_header(&self, did: Did, service_type: impl AsRef<str>) {
        self.inner.configure_proxy_header(format!("{}#{}", did.as_ref(), service_type.as_ref()));
    }
    pub fn clone_with_proxy(&self, did: Did, service_type: impl AsRef<str>) -> Self {
        let cloned = self.clone();
        cloned.inner.configure_proxy_header(format!("{}#{}", did.as_ref(), service_type.as_ref()));
        cloned
    }
    pub fn configure_labelers_header(&self, labeler_dids: Option<Vec<(Did, bool)>>) {
        self.inner.configure_labelers_header(labeler_dids);
    }
    pub async fn get_labelers_header(&self) -> Option<Vec<String>> {
        self.inner.atproto_accept_labelers_header().await
    }
    pub async fn get_proxy_header(&self) -> Option<String> {
        self.inner.atproto_proxy_header().await
    }
    // Internal helper to refresh sessions
    // - Wraps the actual implementation to ensure only one refresh is attempted at a time.
    async fn refresh_session(&self) {
        {
            let mut is_refreshing = self.is_refreshing.lock().await;
            if *is_refreshing {
                drop(is_refreshing);
                return self.notify.notified().await;
            }
            *is_refreshing = true;
        }
        // TODO: Ensure `is_refreshing` is reliably set to false even in the event of unexpected errors within `refresh_session_inner()`.
        self.refresh_session_inner().await;
        *self.is_refreshing.lock().await = false;
        self.notify.notify_waiters();
    }
    async fn refresh_session_inner(&self) {
        if let Ok(output) = self.call_refresh_session().await {
            if let Ok(Some(mut session)) = self.store.get(&()).await {
                session.access_jwt = output.data.access_jwt;
                session.did = output.data.did;
                session.did_doc = output.data.did_doc.clone();
                session.handle = output.data.handle;
                session.refresh_jwt = output.data.refresh_jwt;
                let _ = self.store.set((), session).await;
            }
            if let Some(did_doc) = output
                .data
                .did_doc
                .as_ref()
                .and_then(|value| DidDocument::try_from_unknown(value.clone()).ok())
            {
                self.store.update_endpoint(&did_doc);
            }
        } else {
            let _ = self.store.clear().await;
        }
    }
    // same as `crate::client::com::atproto::server::Service::refresh_session()`
    async fn call_refresh_session(
        &self,
    ) -> Result<
        crate::com::atproto::server::refresh_session::Output,
        crate::com::atproto::server::refresh_session::Error,
    > {
        let response = self
            .inner
            .send_xrpc::<(), (), _, _>(&XrpcRequest {
                method: Method::POST,
                nsid: crate::com::atproto::server::refresh_session::NSID.into(),
                parameters: None,
                input: None,
                encoding: None,
            })
            .await?;
        match response {
            OutputDataOrBytes::Data(data) => Ok(data),
            _ => Err(Error::UnexpectedResponseType),
        }
    }
    fn is_expired<O, E>(result: &Result<OutputDataOrBytes<O>, E>) -> bool
    where
        O: DeserializeOwned + Send + Sync,
        E: DeserializeOwned + Send + Sync + Debug,
    {
        if let Err(Error::XrpcResponse(response)) = &result {
            if let Some(XrpcErrorKind::Undefined(body)) = &response.error {
                if let Some("ExpiredToken") = &body.error.as_deref() {
                    return true;
                }
            }
        }
        false
    }
}

impl<S, T> Clone for Client<S, T>
where
    S: StoreTrait<(), AtpSession> + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            inner: self.inner.clone(),
            is_refreshing: self.is_refreshing.clone(),
            notify: self.notify.clone(),
        }
    }
}

impl<S, T> HttpClient for Client<S, T>
where
    S: Send + Sync,
    T: HttpClient + Send + Sync,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> core::result::Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>>
    {
        self.inner.send_http(request).await
    }
}

impl<S, T> XrpcClient for Client<S, T>
where
    S: StoreTrait<(), AtpSession> + Send + Sync,
    S::Error: Send + Sync + 'static,
    T: XrpcClient + Send + Sync,
{
    fn base_uri(&self) -> String {
        self.inner.base_uri()
    }
    async fn send_xrpc<P, I, O, E>(
        &self,
        request: &XrpcRequest<P, I>,
    ) -> Result<OutputDataOrBytes<O>, E>
    where
        P: Serialize + Send + Sync,
        I: Serialize + Send + Sync,
        O: DeserializeOwned + Send + Sync,
        E: DeserializeOwned + Send + Sync + Debug,
    {
        let result = self.inner.send_xrpc(request).await;
        // handle session-refreshes as needed
        if Self::is_expired(&result) {
            self.refresh_session().await;
            self.inner.send_xrpc(request).await
        } else {
            result
        }
    }
}

pub struct Store<S> {
    inner: S,
    endpoint: RwLock<String>,
}

impl<S> Store<S> {
    pub fn new(inner: S, initial_endpoint: String) -> Self {
        Self { inner, endpoint: RwLock::new(initial_endpoint) }
    }
    pub fn get_endpoint(&self) -> String {
        self.endpoint.read().expect("failed to read endpoint").clone()
    }
    pub fn update_endpoint(&self, did_doc: &DidDocument) {
        if let Some(endpoint) = did_doc.get_pds_endpoint() {
            *self.endpoint.write().expect("failed to write endpoint") = endpoint;
        }
    }
}

impl<S, K, V> StoreTrait<K, V> for Store<S>
where
    K: Eq + Hash + Send + Sync,
    V: Clone + Send,
    S: StoreTrait<K, V> + Sync,
{
    type Error = S::Error;

    async fn get(&self, key: &K) -> core::result::Result<Option<V>, Self::Error> {
        self.inner.get(key).await
    }
    async fn set(&self, key: K, value: V) -> core::result::Result<(), Self::Error> {
        self.inner.set(key, value).await
    }
    async fn del(&self, key: &K) -> core::result::Result<(), Self::Error> {
        self.inner.del(key).await
    }
    async fn clear(&self) -> core::result::Result<(), Self::Error> {
        self.inner.clear().await
    }
}
