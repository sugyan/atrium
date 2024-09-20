#![doc = "XrpcClient implementation for [reqwest]"]
use atrium_xrpc::http::{Request, Response};
use atrium_xrpc::{HttpClient, XrpcClient};
use reqwest::Client;

/// A [`reqwest`] based asynchronous client to make XRPC requests with.
///
/// To change the [`reqwest::Client`] used internally to a custom configured one,
/// use the [`ReqwestClientBuilder`].
///
/// You do **not** have to wrap the `Client` in an [`Rc`] or [`Arc`] to **reuse** it,
/// because it already uses an [`Arc`] internally.
///
/// [`Rc`]: std::rc::Rc
#[derive(Clone)]
pub struct ReqwestClient {
    base_uri: String,
    client: Client,
}

impl ReqwestClient {
    /// Create a new [`ReqwestClient`] using the default configuration.
    pub fn new(base_uri: impl AsRef<str>) -> ReqwestClient {
        ReqwestClientBuilder::new(base_uri).build()
    }
}

/// A client builder, capable of creating custom [`ReqwestClient`] instances.
pub struct ReqwestClientBuilder {
    base_uri: String,
    client: Option<Client>,
}

impl ReqwestClientBuilder {
    /// Create a new [`ReqwestClientBuilder`] for building a custom client.
    pub fn new(base_uri: impl AsRef<str>) -> Self {
        Self { base_uri: base_uri.as_ref().into(), client: None }
    }
    /// Sets the [`reqwest::Client`] to use.
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }
    /// Build an [`ReqwestClient`] using the configured options.
    pub fn build(self) -> ReqwestClient {
        ReqwestClient { base_uri: self.base_uri, client: self.client.unwrap_or_default() }
    }
}

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
        builder.body(response.bytes().await?.to_vec()).map_err(Into::into)
    }
}

impl XrpcClient for ReqwestClient {
    fn base_uri(&self) -> String {
        self.base_uri.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(target_arch = "wasm32"))]
    use std::time::Duration;
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn new() -> Result<(), Box<dyn std::error::Error>> {
        let client = ReqwestClient::new("http://localhost:8080");
        assert_eq!(client.base_uri(), "http://localhost:8080");
        Ok(())
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn builder_without_client() -> Result<(), Box<dyn std::error::Error>> {
        let client = ReqwestClientBuilder::new("http://localhost:8080").build();
        assert_eq!(client.base_uri(), "http://localhost:8080");
        Ok(())
    }

    // TODO: Reqwest::Client doesn't have a `timeout` in wasm module
    // https://github.com/seanmonstar/reqwest/pull/1760
    #[cfg(not(target_arch = "wasm32"))]
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
