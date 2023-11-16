//! An ATP "Agent".
//! Manages session token lifecycles and provides all XRPC methods.
pub mod store;

use self::store::SessionStore;
use crate::client::Service;
use async_trait::async_trait;
use atrium_xrpc::error::{Error, XrpcErrorKind};
use atrium_xrpc::{HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest, XrpcResult};
use http::{Method, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

/// Type alias for the [com::atproto::server::create_session::Output](crate::com::atproto::server::create_session::Output)
pub type Session = crate::com::atproto::server::create_session::Output;

const REFRESH_SESSION: &str = "com.atproto.server.refreshSession";

pub struct SessionAuthWrapper<S, T> {
    store: Arc<S>,
    inner: Arc<T>,
}

#[async_trait]
impl<S, T> HttpClient for SessionAuthWrapper<S, T>
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

#[async_trait]
impl<S, T> XrpcClient for SessionAuthWrapper<S, T>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    fn base_uri(&self) -> &str {
        self.inner.base_uri()
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

pub struct RefreshWrapper<S, T> {
    store: Arc<S>,
    inner: Arc<T>,
    is_refreshing: Arc<Mutex<bool>>,
    notify: Arc<Notify>,
}

impl<S, T> RefreshWrapper<S, T>
where
    S: SessionStore,
    T: XrpcClient + Send + Sync,
{
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
                session.did_doc = output.did_doc;
                session.handle = output.handle;
                session.refresh_jwt = output.refresh_jwt;
                self.store.set_session(session).await;
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

#[async_trait]
impl<S, T> HttpClient for RefreshWrapper<S, T>
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

#[async_trait]
impl<S, T> XrpcClient for RefreshWrapper<S, T>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    fn base_uri(&self) -> &str {
        self.inner.base_uri()
    }
    async fn auth(&self, is_refresh: bool) -> Option<String> {
        self.inner.auth(is_refresh).await
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

type Api<S, T> = Service<RefreshWrapper<S, SessionAuthWrapper<S, T>>>;

pub struct AtpAgent<S, T>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    store: Arc<S>,
    pub api: Api<S, T>,
}

impl<S, T> AtpAgent<S, T>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    pub fn new(xrpc: T, store: S) -> Self {
        let store = Arc::new(store);
        let api = Service::new(Arc::new(RefreshWrapper {
            store: Arc::clone(&store),
            inner: Arc::new(SessionAuthWrapper {
                store: Arc::clone(&store),
                inner: Arc::new(xrpc),
            }),
            is_refreshing: Arc::new(Mutex::new(false)),
            notify: Arc::new(Notify::new()),
        }));
        Self { api, store }
    }
    pub async fn get_session(&self) -> Option<Session> {
        self.store.get_session().await
    }
    /// Start a new session with this agent.
    pub async fn login(
        &self,
        identifier: &str,
        password: &str,
    ) -> Result<Session, Error<crate::com::atproto::server::create_session::Error>> {
        let result = self
            .api
            .com
            .atproto
            .server
            .create_session(crate::com::atproto::server::create_session::Input {
                identifier: identifier.into(),
                password: password.into(),
            })
            .await?;
        self.store.set_session(result.clone()).await;
        Ok(result)
    }
    /// Resume a pre-existing session with this agent.
    pub async fn resume_session(
        &self,
        session: Session,
    ) -> Result<(), Error<crate::com::atproto::server::get_session::Error>> {
        self.store.set_session(session.clone()).await;
        let result = self.api.com.atproto.server.get_session().await;
        match result {
            Ok(output) => {
                assert_eq!(output.did, session.did);
                if let Some(mut session) = self.get_session().await {
                    session.did_doc = output.did_doc;
                    session.email = output.email;
                    session.email_confirmed = output.email_confirmed;
                    session.handle = output.handle;
                    self.store.set_session(session).await;
                }
                Ok(())
            }
            Err(err) => {
                self.store.clear_session().await;
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests;
