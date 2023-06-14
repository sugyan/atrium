#![doc = "Default client implementation using [`reqwest`](https://crates.io/crates/reqwest)."]
use crate::{HttpClient, XrpcClient};
use async_trait::async_trait;
use http::{Request, Response};
use std::error::Error;

#[derive(Debug, Default)]
pub struct Client {
    client: reqwest::Client,
    host: String,
}

impl Client {
    pub fn new(host: String) -> Self {
        Self {
            host,
            ..Default::default()
        }
    }
}

#[async_trait]
impl HttpClient for Client {
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

impl XrpcClient for Client {
    fn host(&self) -> &str {
        &self.host
    }
}
