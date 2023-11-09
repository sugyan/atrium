#![doc = "XrpcClient implementation for [isahc]"]
use async_trait::async_trait;
use atrium_xrpc::{HttpClient, XrpcClient};
use http::{Request, Response};
use isahc::{AsyncReadResponseExt, HttpClient as Client};
use std::sync::Arc;

pub struct IsahcClient {
    host: String,
    client: Arc<Client>,
}

impl IsahcClient {
    pub fn new(host: impl AsRef<str>) -> Self {
        IsahcClientBuilder::new(host).build()
    }
}

pub struct IsahcClientBuilder {
    host: String,
    client: Option<Client>,
}

impl IsahcClientBuilder {
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
    pub fn build(self) -> IsahcClient {
        IsahcClient {
            host: self.host,
            client: Arc::new(
                self.client
                    .unwrap_or(Client::new().expect("failed to create isahc client")),
            ),
        }
    }
}

#[async_trait]
impl HttpClient for IsahcClient {
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut response = self.client.send_async(request).await?;
        let mut builder = Response::builder().status(response.status());
        for (k, v) in response.headers() {
            builder = builder.header(k, v);
        }
        builder
            .body(response.bytes().await?.to_vec())
            .map_err(Into::into)
    }
}

impl XrpcClient for IsahcClient {
    fn host(&self) -> &str {
        &self.host
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use isahc::config::Configurable;
    use std::time::Duration;

    #[test]
    fn new() -> Result<(), Box<dyn std::error::Error>> {
        let client = IsahcClient::new("http://localhost:8080");
        assert_eq!(client.host(), "http://localhost:8080");
        Ok(())
    }

    #[test]
    fn builder_without_client() -> Result<(), Box<dyn std::error::Error>> {
        let client = IsahcClientBuilder::new("http://localhost:8080").build();
        assert_eq!(client.host(), "http://localhost:8080");
        Ok(())
    }

    #[test]
    fn builder_with_client() -> Result<(), Box<dyn std::error::Error>> {
        let client = IsahcClientBuilder::new("http://localhost:8080")
            .client(
                Client::builder()
                    .default_header(http::header::USER_AGENT, "USER_AGENT")
                    .timeout(Duration::from_millis(500))
                    .build()?,
            )
            .build();
        assert_eq!(client.host(), "http://localhost:8080");
        Ok(())
    }
}
