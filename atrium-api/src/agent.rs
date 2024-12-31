pub mod atp_agent;
#[cfg(feature = "bluesky")]
pub mod bluesky;
mod inner;
mod session_manager;

pub use self::session_manager::SessionManager;
use crate::{client::Service, did_doc::DidDocument, types::string::Did};
use atrium_common::store::Store;
use atrium_xrpc::{types::AuthorizationToken, HttpClient, XrpcClient};
use http::{Request, Response};
use std::{
    future::Future,
    marker::PhantomData,
    sync::{Arc, RwLock},
};

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait AuthorizationProvider {
    #[allow(unused_variables)]
    fn authorization_token(
        &self,
        is_refresh: bool,
    ) -> impl Future<Output = Option<AuthorizationToken>>;
}

/// Supported proxy targets.
#[cfg(feature = "bluesky")]
pub type AtprotoServiceType = self::bluesky::AtprotoServiceType;

#[cfg(not(feature = "bluesky"))]
pub enum AtprotoServiceType {
    AtprotoLabeler,
}

#[cfg(not(feature = "bluesky"))]
impl AsRef<str> for AtprotoServiceType {
    fn as_ref(&self) -> &str {
        match self {
            Self::AtprotoLabeler => "atproto_labeler",
        }
    }
}

pub struct Agent<M>
where
    M: SessionManager + Send + Sync,
{
    session_manager: Arc<inner::Wrapper<M>>,
    pub api: Service<inner::Wrapper<M>>,
}

impl<M> Agent<M>
where
    M: SessionManager + Send + Sync,
{
    pub fn new(session_manager: M) -> Self {
        let session_manager = Arc::new(inner::Wrapper::new(session_manager));
        let api = Service::new(session_manager.clone());
        Self { session_manager, api }
    }
    pub async fn did(&self) -> Option<Did> {
        self.session_manager.did().await
    }
}

pub struct WrapperClient<S, T, U> {
    store: Arc<InnerStore<S, U>>,
    proxy_header: RwLock<Option<String>>,
    labelers_header: Arc<RwLock<Option<Vec<String>>>>,
    inner: Arc<T>,
}

impl<S, T, U> WrapperClient<S, T, U>
where
    S: Store<(), U>,
    U: Clone,
{
    pub fn new(store: Arc<InnerStore<S, U>>, xrpc: T) -> Self {
        Self {
            store: Arc::clone(&store),
            labelers_header: Arc::new(RwLock::new(None)),
            proxy_header: RwLock::new(None),
            inner: Arc::new(xrpc),
        }
    }
    pub fn configure_proxy_header(&self, value: String) {
        self.proxy_header.write().expect("failed to write proxy header").replace(value);
    }
    pub fn configure_labelers_header(&self, labelers_dids: Option<Vec<(Did, bool)>>) {
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

impl<S, T, U> Clone for WrapperClient<S, T, U> {
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

impl<S, T, U> HttpClient for WrapperClient<S, T, U>
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

impl<S, T, U> XrpcClient for WrapperClient<S, T, U>
where
    S: Store<(), U> + AuthorizationProvider + Send + Sync,
    T: XrpcClient + Send + Sync,
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

pub struct InnerStore<S, U> {
    inner: S,
    endpoint: RwLock<String>,
    _phantom: PhantomData<U>,
}

impl<S, U> InnerStore<S, U> {
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

impl<S, U> InnerStore<S, U>
where
    S: Store<(), U>,
    U: Clone,
{
    async fn get(&self) -> Result<Option<U>, S::Error> {
        self.inner.get(&()).await
    }
    async fn set(&self, value: U) -> Result<(), S::Error> {
        self.inner.set((), value).await
    }
    async fn clear(&self) -> Result<(), S::Error> {
        self.inner.clear().await
    }
}

impl<S, U> AuthorizationProvider for InnerStore<S, U>
where
    S: Store<(), U> + AuthorizationProvider + Send + Sync,
    U: Clone + Send + Sync,
{
    async fn authorization_token(&self, is_refresh: bool) -> Option<AuthorizationToken> {
        self.inner.authorization_token(is_refresh).await
    }
}
