#![doc = "Default client implementation."]
use crate::{HttpClient, XrpcClient};
use async_trait::async_trait;
use http::{Request, Response};
use reqwest::Client;
use std::error::Error;

#[derive(Debug, Default)]
pub struct XrpcReqwestClient {
    client: Client,
    host: String,
}

impl XrpcReqwestClient {
    pub fn new(host: String) -> Self {
        Self {
            host,
            ..Default::default()
        }
    }
}

#[async_trait]
impl HttpClient for XrpcReqwestClient {
    async fn send(
        &self,
        req: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn Error + Send + Sync + 'static>> {
        let res = self.client.execute(req.try_into()?).await?;
        let mut builder = Response::builder().status(res.status());
        for (k, v) in res.headers() {
            builder = builder.header(k, v);
        }
        builder
            .body(res.bytes().await?.to_vec())
            .map_err(Into::into)
    }
}

impl XrpcClient for XrpcReqwestClient {
    fn host(&self) -> &str {
        &self.host
    }
}
