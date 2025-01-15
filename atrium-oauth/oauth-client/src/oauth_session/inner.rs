use super::store::OAuthSessionStore;
use atrium_api::{
    agent::{CloneWithProxy, Configure, InnerStore, WrapperClient},
    types::string::Did,
};
use atrium_xrpc::{
    http::{Request, Response},
    Error, HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, sync::Arc};

pub struct Client<S, T> {
    inner: WrapperClient<S, T, String>,
}

impl<S, T> Client<S, T> {
    pub fn new(store: Arc<InnerStore<S, String>>, xrpc: T) -> Self {
        Self { inner: WrapperClient::new(Arc::clone(&store), xrpc) }
    }
    async fn refresh_token(&self) {}
    // https://datatracker.ietf.org/doc/html/rfc6750#section-3
    // fn is_invalid_token_response<O, E>(result: &Result<OutputDataOrBytes<O>, Error<E>>) -> bool
    // where
    //     O: DeserializeOwned + Send + Sync,
    //     E: DeserializeOwned + Send + Sync + Debug,
    // {
    //     todo!()
    // }
}

impl<S, T> HttpClient for Client<S, T>
where
    S: OAuthSessionStore + Send + Sync,
    T: HttpClient + Send + Sync,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.inner.send_http(request).await
    }
}

impl<S, T> XrpcClient for Client<S, T>
where
    S: OAuthSessionStore + Send + Sync,
    T: HttpClient + Send + Sync,
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
        // let result = self.inner.send_xrpc(request).await;
        // // handle session-refreshes as needed
        // if Self::is_invalid_token_response(&result) {
        //     self.refresh_token().await;
        //     self.inner.send_xrpc(request).await
        // } else {
        //     result
        // }
        self.inner.send_xrpc(request).await
    }
}

impl<S, T> Configure for Client<S, T> {
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

impl<S, T> CloneWithProxy for Client<S, T>
where
    WrapperClient<S, T, String>: CloneWithProxy,
{
    fn clone_with_proxy(&self, did: Did, service_type: impl AsRef<str>) -> Self {
        Self { inner: self.inner.clone_with_proxy(did, service_type) }
    }
}
