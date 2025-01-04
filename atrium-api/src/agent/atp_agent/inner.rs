use super::{AtpSession, AtpSessionStore};
use crate::{
    agent::{CloneWithProxy, Configure, InnerStore, WrapperClient},
    did_doc::DidDocument,
    types::{string::Did, TryFromUnknown},
};
use atrium_xrpc::{
    error::{Error, Result, XrpcErrorKind},
    {HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest},
};
use http::{Method, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, sync::Arc};
use tokio::sync::{Mutex, Notify};

pub struct Client<S, T> {
    store: Arc<InnerStore<S, AtpSession>>,
    inner: WrapperClient<S, T, AtpSession>,
    is_refreshing: Arc<Mutex<bool>>,
    notify: Arc<Notify>,
}

impl<S, T> Client<S, T>
where
    S: AtpSessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    pub fn new(store: Arc<InnerStore<S, AtpSession>>, xrpc: T) -> Self {
        let inner = WrapperClient::new(Arc::clone(&store), xrpc);
        Self {
            store,
            inner,
            is_refreshing: Arc::new(Mutex::new(false)),
            notify: Arc::new(Notify::new()),
        }
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
            if let Ok(Some(mut session)) = self.store.get().await {
                session.access_jwt = output.data.access_jwt;
                session.did = output.data.did;
                session.did_doc = output.data.did_doc.clone();
                session.handle = output.data.handle;
                session.refresh_jwt = output.data.refresh_jwt;
                self.store.set(session).await.ok();
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
            self.store.clear().await.ok();
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

impl<S, T> Configure for Client<S, T> {
    fn configure_endpoint(&self, endpoint: String) {
        *self.store.endpoint.write().expect("failed to write endpoint") = endpoint;
    }
    fn configure_labelers_header(&self, labeler_dids: Option<Vec<(Did, bool)>>) {
        self.inner.configure_labelers_header(labeler_dids);
    }
    fn configure_proxy_header(&self, did: Did, service_type: impl AsRef<str>) {
        self.inner.configure_proxy_header(did, service_type);
    }
}

impl<S, T> CloneWithProxy for Client<S, T> {
    fn clone_with_proxy(&self, did: Did, service_type: impl AsRef<str>) -> Self {
        let cloned = self.clone();
        cloned.inner.configure_proxy_header(did, service_type);
        cloned
    }
}

impl<S, T> Clone for Client<S, T> {
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
    S: AtpSessionStore + Send + Sync,
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
    S: AtpSessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
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
