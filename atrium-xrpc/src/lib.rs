use async_trait::async_trait;
use atrium_api::xrpc::{HttpClient, XrpcClient};
use http::{Request, Response};
use reqwest::Client;
use std::error::Error;

#[derive(Debug, Default)]
pub struct XrpcReqwestClient {
    client: Client,
    auth: Option<String>,
    host: String,
}

impl XrpcReqwestClient {
    pub fn new(host: String) -> Self {
        Self {
            host,
            ..Default::default()
        }
    }
    pub fn set_auth(&mut self, auth: String) {
        self.auth = Some(auth);
    }
}

#[async_trait]
impl HttpClient for XrpcReqwestClient {
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Box<dyn Error>> {
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
    fn auth(&self) -> Option<&str> {
        self.auth.as_deref()
    }
}

atrium_api::impl_traits!(XrpcReqwestClient);
