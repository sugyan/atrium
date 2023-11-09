#![doc = "XrpcClient implementation for [reqwest]"]
use async_trait::async_trait;
use atrium_xrpc::{HttpClient, XrpcClient};
use http::{Request, Response};
use reqwest::Client;
use std::sync::Arc;

pub struct ReqwestClient {
    base_uri: String,
    client: Arc<Client>,
}

impl ReqwestClient {
    pub fn new(base_uri: impl AsRef<str>) -> ReqwestClient {
        ReqwestClientBuilder::new(base_uri).build()
    }
}

pub struct ReqwestClientBuilder {
    base_uri: String,
    client: Option<Client>,
}

impl ReqwestClientBuilder {
    pub fn new(base_uri: impl AsRef<str>) -> Self {
        Self {
            base_uri: base_uri.as_ref().into(),
            client: None,
        }
    }
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }
    pub fn build(self) -> ReqwestClient {
        ReqwestClient {
            base_uri: self.base_uri,
            client: Arc::new(self.client.unwrap_or_default()),
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
    fn base_uri(&self) -> &str {
        &self.base_uri
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn new() -> Result<(), Box<dyn std::error::Error>> {
        let client = ReqwestClient::new("http://localhost:8080");
        assert_eq!(client.base_uri(), "http://localhost:8080");
        Ok(())
    }

    #[test]
    fn builder_without_client() -> Result<(), Box<dyn std::error::Error>> {
        let client = ReqwestClientBuilder::new("http://localhost:8080").build();
        assert_eq!(client.base_uri(), "http://localhost:8080");
        Ok(())
    }

    #[test]
    fn builder_with_client() -> Result<(), Box<dyn std::error::Error>> {
        let client = ReqwestClientBuilder::new("http://localhost:8080")
            .client(
                Client::builder()
                    .user_agent("USER_AGENT")
                    .timeout(Duration::from_millis(500))
                    .build()?,
            )
            .build();
        assert_eq!(client.base_uri(), "http://localhost:8080");
        Ok(())
    }
}
