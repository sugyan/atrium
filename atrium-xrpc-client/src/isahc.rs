#![doc = "XrpcClient implementation for [isahc]"]
use async_trait::async_trait;
use atrium_xrpc::{HttpClient, XrpcClient};
use http::{Request, Response};
use isahc::{AsyncReadResponseExt, HttpClient as Client};
use std::sync::Arc;

/// A [`isahc`] based asynchronous client to make XRPC requests with.
///
/// To change the [`isahc::HttpClient`] used internally to a custom configured one,
/// use the [`IsahcClientBuilder`].
///
/// You do **not** have to wrap the `Client` in an [`Rc`] or [`Arc`] to **reuse** it,
/// because it already uses an [`Arc`] internally.
///
/// [`Rc`]: std::rc::Rc
pub struct IsahcClient {
    base_uri: String,
    client: Arc<Client>,
}

impl IsahcClient {
    /// Create a new [`IsahcClient`] using the default configuration.
    pub fn new(base_uri: impl AsRef<str>) -> Self {
        IsahcClientBuilder::new(base_uri).build()
    }
}

/// A client builder, capable of creating custom [`IsahcClient`] instances.
pub struct IsahcClientBuilder {
    base_uri: String,
    client: Option<Client>,
}

impl IsahcClientBuilder {
    /// Create a new [`IsahcClientBuilder`] for building a custom client.
    pub fn new(base_uri: impl AsRef<str>) -> Self {
        Self {
            base_uri: base_uri.as_ref().into(),
            client: None,
        }
    }
    /// Sets the [`isahc::HttpClient`] to use.
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }
    /// Build an [`IsahcClient`] using the configured options.
    pub fn build(self) -> IsahcClient {
        IsahcClient {
            base_uri: self.base_uri,
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
    fn base_uri(&self) -> &str {
        &self.base_uri
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
        assert_eq!(client.base_uri(), "http://localhost:8080");
        Ok(())
    }

    #[test]
    fn builder_without_client() -> Result<(), Box<dyn std::error::Error>> {
        let client = IsahcClientBuilder::new("http://localhost:8080").build();
        assert_eq!(client.base_uri(), "http://localhost:8080");
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
        assert_eq!(client.base_uri(), "http://localhost:8080");
        Ok(())
    }
}
