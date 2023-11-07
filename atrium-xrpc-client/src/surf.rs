#![doc = "XrpcClient implementation for [surf]"]
use async_trait::async_trait;
use atrium_xrpc::{HttpClient, XrpcClient};
use http::{Request, Response};
use std::sync::Arc;
use surf::Client;

pub struct SurfClient {
    host: String,
    client: Arc<Client>,
}

impl SurfClient {
    pub fn new(host: impl AsRef<str>, client: Client) -> Self {
        Self {
            host: host.as_ref().to_string(),
            client: Arc::new(client),
        }
    }
}

#[async_trait]
impl HttpClient for SurfClient {
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let method = match *request.method() {
            http::Method::GET => surf::http::Method::Get,
            http::Method::POST => surf::http::Method::Post,
            _ => unimplemented!(),
        };
        let url = surf::http::Url::parse(&request.uri().to_string())?;
        let mut req_builder = surf::RequestBuilder::new(method, url);
        for (name, value) in request.headers() {
            req_builder = req_builder.header(name.as_str(), value.to_str()?);
        }
        let mut response = self
            .client
            .send(req_builder.body(request.body().to_vec()).build())
            .await?;
        let mut res_builder = Response::builder();
        for (name, values) in response.iter() {
            for value in values {
                res_builder = res_builder.header(name.as_str(), value.as_str());
            }
        }
        res_builder
            .status(u16::from(response.status()))
            .body(response.body_bytes().await?)
            .map_err(Into::into)
    }
}

impl XrpcClient for SurfClient {
    fn host(&self) -> &str {
        &self.host
    }
}
