pub mod agent;
mod error;
pub mod moderation;
pub mod preference;
#[cfg(feature = "rich-text")]
pub mod rich_text;

pub use agent::BskyAgent;
pub use atrium_api as api;
pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use atrium_api::xrpc::http::{Request, Response};
    use atrium_api::xrpc::types::Header;
    use atrium_api::xrpc::{HttpClient, XrpcClient};

    pub struct MockClient;

    #[async_trait]
    impl HttpClient for MockClient {
        async fn send_http(
            &self,
            request: Request<Vec<u8>>,
        ) -> core::result::Result<
            Response<Vec<u8>>,
            Box<dyn std::error::Error + Send + Sync + 'static>,
        > {
            if let Some(handle) = request
                .uri()
                .query()
                .and_then(|s| s.strip_prefix("handle="))
            {
                Ok(Response::builder()
                    .status(200)
                    .header(Header::ContentType, "application/json")
                    .body(
                        format!(r#"{{"did": "did:fake:{}"}}"#, handle)
                            .as_bytes()
                            .to_vec(),
                    )?)
            } else {
                Ok(Response::builder().status(500).body(Vec::new())?)
            }
        }
    }

    #[async_trait]
    impl XrpcClient for MockClient {
        fn base_uri(&self) -> String {
            String::new()
        }
    }
}
