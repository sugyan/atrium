#![doc = "Default client implementation using [`reqwest`](https://crates.io/crates/reqwest)."]
use crate::{HttpClient, XrpcClient};
use async_trait::async_trait;
use http::{Request, Response};
use std::error::Error;

#[derive(Debug, Default)]
pub struct ReqwestClient {
    client: reqwest::Client,
    base_uri: String,
}

impl ReqwestClient {
    pub fn new(base_uri: String) -> Self {
        Self {
            base_uri,
            ..Default::default()
        }
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn send_http(
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

impl XrpcClient for ReqwestClient {
    fn base_uri(&self) -> &str {
        &self.base_uri
    }
}
