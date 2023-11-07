use async_trait::async_trait;
use atrium_xrpc::{HttpClient, XrpcClient};
use http::{Request, Response};
use reqwest::Client;
use std::sync::Arc;

pub struct ReqwestClient {
    host: String,
    client: Arc<Client>,
}

impl ReqwestClient {
    pub fn new(host: impl AsRef<str>) -> ReqwestClient {
        ReqwestClientBuilder::new(host).build()
    }
}

pub struct ReqwestClientBuilder {
    host: String,
    client: Option<Client>,
}

impl ReqwestClientBuilder {
    pub fn new(host: impl AsRef<str>) -> Self {
        Self {
            host: host.as_ref().into(),
            client: None,
        }
    }
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }
    pub fn build(self) -> ReqwestClient {
        ReqwestClient {
            host: self.host,
            client: Arc::new(self.client.unwrap_or_else(Client::new)),
        }
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let response = self.client.execute(request.try_into()?).await?;
        let mut builder = Response::builder().status(response.status());
        for (k, v) in response.headers() {
            builder = builder.header(k, v);
        }
        builder
            .body(response.bytes().await?.to_vec())
            .map_err(Into::into)
    }
}

impl XrpcClient for ReqwestClient {
    fn host(&self) -> &str {
        &self.host
    }
}
