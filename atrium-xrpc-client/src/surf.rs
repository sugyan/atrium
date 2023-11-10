#![doc = "XrpcClient implementation for [surf]"]
use async_trait::async_trait;
use atrium_xrpc::{HttpClient, XrpcClient};
use http::{Request, Response};
use std::sync::Arc;
use surf::Client;

/// A [`surf`] based asynchronous client to make XRPC requests with.
///
/// You do **not** have to wrap the `Client` in an [`Rc`] or [`Arc`] to **reuse** it,
/// because it already uses an [`Arc`] internally.
///
/// [`Rc`]: std::rc::Rc
pub struct SurfClient {
    base_uri: String,
    client: Arc<Client>,
}

impl SurfClient {
    /// Create a new [`SurfClient`] using the passed [`surf::Client`].
    pub fn new(base_uri: impl AsRef<str>, client: Client) -> Self {
        Self {
            base_uri: base_uri.as_ref().to_string(),
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
    fn base_uri(&self) -> &str {
        &self.base_uri
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_client::h1::H1Client;
    use std::time::Duration;

    #[test]
    fn new() -> Result<(), Box<dyn std::error::Error>> {
        let client = SurfClient::new(
            "http://localhost:8080",
            Client::try_from(
                surf::Config::default()
                    .set_http_client(H1Client::try_from(
                        http_client::Config::default()
                            .set_timeout(Some(Duration::from_millis(500))),
                    )?)
                    .add_header(surf::http::headers::USER_AGENT, "USER_AGENT")?,
            )?,
        );
        assert_eq!(client.base_uri(), "http://localhost:8080");
        Ok(())
    }
}
