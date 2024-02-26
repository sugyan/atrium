use super::{Session, SessionStore};
use crate::did_doc::DidDocument;
use async_trait::async_trait;
use atrium_xrpc::error::{Error, XrpcErrorKind};
use atrium_xrpc::{HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest, XrpcResult};
use http::{Method, Request, Response, Uri};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{Arc, RwLock};
use tokio::sync::{Mutex, Notify};

const REFRESH_SESSION: &str = "com.atproto.server.refreshSession";

struct SessionStoreClient<S, T> {
    store: Arc<Store<S>>,
    inner: T,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<S, T> HttpClient for SessionStoreClient<S, T>
where
    S: Send + Sync,
    T: HttpClient + Send + Sync,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.inner.send_http(request).await
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<S, T> XrpcClient for SessionStoreClient<S, T>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    fn base_uri(&self) -> String {
        self.store.get_endpoint()
    }
    async fn auth(&self, is_refresh: bool) -> Option<String> {
        self.store.get_session().await.map(|session| {
            if is_refresh {
                session.refresh_jwt.clone()
            } else {
                session.access_jwt.clone()
            }
        })
    }
}

pub struct Client<S, T> {
    store: Arc<Store<S>>,
    inner: SessionStoreClient<S, T>,
    is_refreshing: Mutex<bool>,
    notify: Notify,
}

impl<S, T> Client<S, T>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    pub(crate) fn new(store: Arc<Store<S>>, xrpc: T) -> Self {
        let inner = SessionStoreClient {
            store: Arc::clone(&store),
            inner: xrpc,
        };
        Self {
            store,
            inner,
            is_refreshing: Mutex::new(false),
            notify: Notify::new(),
        }
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
            if let Some(mut session) = self.store.get_session().await {
                session.access_jwt = output.access_jwt;
                session.did = output.did;
                session.did_doc = output.did_doc.clone();
                session.handle = output.handle;
                session.refresh_jwt = output.refresh_jwt;
                self.store.set_session(session).await;
            }
            if let Some(did_doc) = &output.did_doc {
                self.store.update_endpoint(did_doc);
            }
        } else {
            self.store.clear_session().await;
        }
    }
    // same as `crate::client::com::atproto::server::Service::refresh_session()`
    async fn call_refresh_session(
        &self,
    ) -> Result<
        crate::com::atproto::server::refresh_session::Output,
        Error<crate::com::atproto::server::refresh_session::Error>,
    > {
        let response = self
            .inner
            .send_xrpc::<(), (), _, _>(&XrpcRequest {
                method: Method::POST,
                path: REFRESH_SESSION.into(),
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
    fn is_expired<O, E>(result: &XrpcResult<O, E>) -> bool
    where
        O: DeserializeOwned + Send + Sync,
        E: DeserializeOwned + Send + Sync,
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

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<S, T> HttpClient for Client<S, T>
where
    S: Send + Sync,
    T: HttpClient + Send + Sync,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.inner.send_http(request).await
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<S, T> XrpcClient for Client<S, T>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    fn base_uri(&self) -> String {
        self.inner.base_uri()
    }
    async fn send_xrpc<P, I, O, E>(&self, request: &XrpcRequest<P, I>) -> XrpcResult<O, E>
    where
        P: Serialize + Send + Sync,
        I: Serialize + Send + Sync,
        O: DeserializeOwned + Send + Sync,
        E: DeserializeOwned + Send + Sync,
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
        Self {
            inner,
            endpoint: RwLock::new(initial_endpoint),
        }
    }
    pub fn get_endpoint(&self) -> String {
        self.endpoint
            .read()
            .expect("failed to read endpoint")
            .clone()
    }
    pub fn update_endpoint(&self, did_doc: &DidDocument) {
        if let Some(endpoint) = Self::get_pds_endpoint(did_doc) {
            *self.endpoint.write().expect("failed to write endpoint") = endpoint;
        }
    }
    fn get_pds_endpoint(did_doc: &DidDocument) -> Option<String> {
        Self::get_service_endpoint(did_doc, ("#atproto_pds", "AtprotoPersonalDataServer"))
    }
    fn get_service_endpoint(did_doc: &DidDocument, (id, r#type): (&str, &str)) -> Option<String> {
        let full_id = did_doc.id.clone() + id;
        if let Some(services) = &did_doc.service {
            let service = services
                .iter()
                .find(|service| service.id == id || service.id == full_id)?;
            if service.r#type == r#type && Self::validate_url(&service.service_endpoint) {
                return Some(service.service_endpoint.clone());
            }
        }
        None
    }
    fn validate_url(url: &str) -> bool {
        if let Ok(uri) = url.parse::<Uri>() {
            if let Some(scheme) = uri.scheme() {
                if (scheme == "https" || scheme == "http") && uri.host().is_some() {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<S> SessionStore for Store<S>
where
    S: SessionStore + Send + Sync,
{
    async fn get_session(&self) -> Option<Session> {
        self.inner.get_session().await
    }
    async fn set_session(&self, session: Session) {
        self.inner.set_session(session).await;
    }
    async fn clear_session(&self) {
        self.inner.clear_session().await;
    }
}
