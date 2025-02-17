//! Utilities for managing sessions and endpoints.

use super::{AuthorizationProvider, CloneWithProxy, Configure};
use crate::{did_doc::DidDocument, types::string::Did};
use atrium_common::store::Store;
use atrium_xrpc::{types::AuthorizationToken, HttpClient, XrpcClient};
use http::{Request, Response};
use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};

/// A client that maintains session data and manages endpoints and XRPC headers.  
///
/// It is recommended to use this struct internally in higher-level clients such as [`XrpcClient`], which can automatically update tokens.
pub struct SessionClient<S, T, U> {
    store: Arc<SessionWithEndpointStore<S, U>>,
    proxy_header: RwLock<Option<String>>,
    labelers_header: Arc<RwLock<Option<Vec<String>>>>,
    inner: Arc<T>,
}

impl<S, T, U> SessionClient<S, T, U> {
    pub fn new(store: Arc<SessionWithEndpointStore<S, U>>, http_client: T) -> Self {
        Self {
            store: Arc::clone(&store),
            labelers_header: Arc::new(RwLock::new(None)),
            proxy_header: RwLock::new(None),
            inner: Arc::new(http_client),
        }
    }
}

impl<S, T, U> Configure for SessionClient<S, T, U> {
    fn configure_endpoint(&self, endpoint: String) {
        *self.store.endpoint.write().expect("failed to write endpoint") = endpoint;
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
    fn configure_proxy_header(&self, did: Did, service_type: impl AsRef<str>) {
        self.proxy_header.write().expect("failed to write proxy header").replace(format!(
            "{}#{}",
            did.as_ref(),
            service_type.as_ref()
        ));
    }
}

impl<S, T, U> CloneWithProxy for SessionClient<S, T, U> {
    fn clone_with_proxy(&self, did: Did, service_type: impl AsRef<str>) -> Self {
        let cloned = self.clone();
        cloned.configure_proxy_header(did, service_type);
        cloned
    }
}

impl<S, T, U> Clone for SessionClient<S, T, U> {
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

impl<S, T, U> HttpClient for SessionClient<S, T, U>
where
    S: Store<(), U> + Send + Sync,
    T: HttpClient + Send + Sync,
    U: Clone + Send + Sync,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> core::result::Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>>
    {
        self.inner.send_http(request).await
    }
}

impl<S, T, U> XrpcClient for SessionClient<S, T, U>
where
    S: Store<(), U> + AuthorizationProvider + Send + Sync,
    T: HttpClient + Send + Sync,
    U: Clone + Send + Sync,
{
    fn base_uri(&self) -> String {
        self.store.get_endpoint()
    }
    async fn authorization_token(&self, is_refresh: bool) -> Option<AuthorizationToken> {
        self.store.authorization_token(is_refresh).await
    }
    async fn atproto_proxy_header(&self) -> Option<String> {
        self.proxy_header.read().expect("failed to read proxy header").clone()
    }
    async fn atproto_accept_labelers_header(&self) -> Option<Vec<String>> {
        self.labelers_header.read().expect("failed to read labelers header").clone()
    }
}

/// A store that wraps an underlying store providing authorization token and adds endpoint management functionality.
///
/// This struct is intended to be used when creating a [`SessionClient`].
pub struct SessionWithEndpointStore<S, U> {
    inner: S,
    pub endpoint: RwLock<String>,
    _phantom: PhantomData<U>,
}

impl<S, U> SessionWithEndpointStore<S, U> {
    pub fn new(inner: S, initial_endpoint: String) -> Self {
        Self { inner, endpoint: RwLock::new(initial_endpoint), _phantom: PhantomData }
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

impl<S, U> SessionWithEndpointStore<S, U>
where
    S: Store<(), U>,
    U: Clone,
{
    pub async fn get(&self) -> Result<Option<U>, S::Error> {
        self.inner.get(&()).await
    }
    pub async fn set(&self, value: U) -> Result<(), S::Error> {
        self.inner.set((), value).await
    }
    pub async fn clear(&self) -> Result<(), S::Error> {
        self.inner.clear().await
    }
}

impl<S, U> AuthorizationProvider for SessionWithEndpointStore<S, U>
where
    S: Store<(), U> + AuthorizationProvider + Send + Sync,
    U: Clone + Send + Sync,
{
    async fn authorization_token(&self, is_refresh: bool) -> Option<AuthorizationToken> {
        self.inner.authorization_token(is_refresh).await
    }
}
