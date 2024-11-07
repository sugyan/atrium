use super::SessionManager;
use crate::types::string::Did;
use atrium_xrpc::{Error, HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest};
use http::{Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, ops::Deref, sync::Arc};

pub struct Wrapper<M>
where
    M: SessionManager + Send + Sync,
{
    inner: Arc<M>,
}

impl<M> Wrapper<M>
where
    M: SessionManager + Send + Sync,
{
    pub fn new(inner: M) -> Self {
        Self { inner: Arc::new(inner) }
    }
}

impl<M> HttpClient for Wrapper<M>
where
    M: SessionManager + Send + Sync,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.inner.send_http(request).await
    }
}

impl<M> XrpcClient for Wrapper<M>
where
    M: SessionManager + Send + Sync,
{
    fn base_uri(&self) -> String {
        self.inner.base_uri()
    }
    // async fn authentication_token(&self, is_refresh: bool) -> Option<String> {
    //     self.inner.authentication_token(is_refresh).await
    // }
    // async fn atproto_proxy_header(&self) -> Option<String> {
    //     self.inner.atproto_proxy_header().await
    // }
    // async fn atproto_accept_labelers_header(&self) -> Option<Vec<String>> {
    //     self.inner.atproto_accept_labelers_header().await
    // }
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
        self.inner.send_xrpc(request).await
    }
}

impl<M> SessionManager for Wrapper<M>
where
    M: SessionManager + Send + Sync,
{
    async fn did(&self) -> Option<Did> {
        self.inner.did().await
    }
}

impl<M> Clone for Wrapper<M>
where
    M: SessionManager + Send + Sync,
{
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<M> Deref for Wrapper<M>
where
    M: SessionManager + Send + Sync,
{
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
