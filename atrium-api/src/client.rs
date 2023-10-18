#![doc = "An ATP service client."]
use crate::client_services::Service;
use async_trait::async_trait;
use atrium_xrpc::error::Error;
use atrium_xrpc::{HttpClient, InputDataOrBytes, OutputDataOrBytes, XrpcClient};
use http::{Method, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

/// Wrapper trait of the [`XrpcClient`] trait.
#[async_trait]
pub trait AtpService: XrpcClient + Send + Sync {
    async fn send<P, I, O, E>(
        &self,
        method: Method,
        path: &str,
        parameters: Option<P>,
        input: Option<InputDataOrBytes<I>>,
        encoding: Option<String>,
    ) -> Result<OutputDataOrBytes<O>, Error<E>>
    where
        P: Serialize + Send,
        I: Serialize + Send,
        O: DeserializeOwned,
        E: DeserializeOwned,
    {
        self.send_xrpc(method, path, parameters, input, encoding)
            .await
    }
}

/// Wrapper struct for [`AtpServiceClient`].
pub struct AtpServiceWrapper<X>
where
    X: XrpcClient + Send + Sync,
{
    xrpc: X,
}

impl<X> AtpServiceWrapper<X>
where
    X: XrpcClient + Send + Sync,
{
    pub fn new(xrpc: X) -> Self {
        Self { xrpc }
    }
}

#[async_trait]
impl<X> HttpClient for AtpServiceWrapper<X>
where
    X: XrpcClient + Send + Sync,
{
    async fn send_http(
        &self,
        req: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        self.xrpc.send_http(req).await
    }
}

impl<X> XrpcClient for AtpServiceWrapper<X>
where
    X: XrpcClient + Send + Sync,
{
    fn host(&self) -> &str {
        self.xrpc.host()
    }
}

impl<X> AtpService for AtpServiceWrapper<X> where X: XrpcClient + Send + Sync {}

/// Client struct for the ATP service.
pub struct AtpServiceClient<T>
where
    T: AtpService,
{
    pub service: Service<T>,
}

impl<X> AtpServiceClient<AtpServiceWrapper<X>>
where
    X: XrpcClient + Send + Sync,
{
    pub fn new(xrpc: X) -> Self {
        Self {
            service: Service::new(Arc::new(AtpServiceWrapper::new(xrpc))),
        }
    }
}
