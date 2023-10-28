#![doc = r#"An ATP service client."#]
use crate::client_services::Service;
use async_trait::async_trait;
use atrium_xrpc::{HttpClient, InputDataOrBytes, OutputDataOrBytes, XrpcClient};
use std::sync::Arc;

/// Wrapper trait of the [`XrpcClient`] trait.
#[async_trait]
pub trait AtpService: XrpcClient + Send + Sync {
    async fn send<P, I, O, E>(
        &self,
        method: http::Method,
        path: &str,
        parameters: Option<P>,
        input: Option<InputDataOrBytes<I>>,
        encoding: Option<String>,
    ) -> Result<OutputDataOrBytes<O>, atrium_xrpc::error::Error<E>>
    where
        P: serde::Serialize + Send,
        I: serde::Serialize + Send,
        O: serde::de::DeserializeOwned,
        E: serde::de::DeserializeOwned,
    {
        self.send_xrpc(method, path, parameters, input, encoding)
            .await
    }
}

/// Wrapper struct for the AtpServiceClient.
pub struct AtpServiceWrapper<T>
where
    T: XrpcClient + Send + Sync,
{
    xrpc: T,
}

impl<T> AtpServiceWrapper<T>
where
    T: XrpcClient + Send + Sync,
{
    pub fn new(xrpc: T) -> Self {
        Self { xrpc }
    }
}

#[async_trait]
impl<T> HttpClient for AtpServiceWrapper<T>
where
    T: XrpcClient + Send + Sync,
{
    async fn send_http(
        &self,
        req: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        self.xrpc.send_http(req).await
    }
}

#[async_trait]
impl<T> XrpcClient for AtpServiceWrapper<T>
where
    T: XrpcClient + Send + Sync,
{
    fn host(&self) -> &str {
        self.xrpc.host()
    }
    fn auth(&self, is_refresh: bool) -> Option<String> {
        self.xrpc.auth(is_refresh)
    }
    async fn send_xrpc<P, I, O, E>(
        &self,
        method: http::Method,
        path: &str,
        parameters: Option<P>,
        input: Option<InputDataOrBytes<I>>,
        encoding: Option<String>,
    ) -> Result<OutputDataOrBytes<O>, atrium_xrpc::error::Error<E>>
    where
        P: serde::Serialize + Send,
        I: serde::Serialize + Send,
        O: serde::de::DeserializeOwned,
        E: serde::de::DeserializeOwned,
    {
        self.xrpc
            .send_xrpc(method, path, parameters, input, encoding)
            .await
    }
}

impl<T> AtpService for AtpServiceWrapper<T> where T: XrpcClient + Send + Sync {}

/// Client struct for the ATP service.
pub struct AtpServiceClient<T>
where
    T: AtpService,
{
    pub service: Service<T>,
}

impl<T> AtpServiceClient<AtpServiceWrapper<T>>
where
    T: XrpcClient + Send + Sync,
{
    pub fn new(xrpc: T) -> Self {
        Self {
            service: Service::new(Arc::new(AtpServiceWrapper::new(xrpc))),
        }
    }
}
