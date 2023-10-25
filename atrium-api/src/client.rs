#![doc = "An ATP service client."]
use crate::client_services::Service;
use atrium_xrpc::XrpcClient;
use std::sync::Arc;

/// Client struct for the ATP service.
pub struct AtpServiceClient<T>
where
    T: XrpcClient + Send + Sync,
{
    pub service: Service<T>,
}

impl<T> AtpServiceClient<T>
where
    T: XrpcClient + Send + Sync,
{
    pub fn new(xrpc: T) -> Self {
        Self {
            service: Service::new(Arc::new(xrpc)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use atrium_xrpc::HttpClient;
    use http::{Request, Response, StatusCode};

    struct DummyXrpcClient;

    #[async_trait]
    impl HttpClient for DummyXrpcClient {
        async fn send_http(
            &self,
            _request: Request<Vec<u8>>,
        ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
            Ok(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(serde_json::to_vec(
                    &atrium_xrpc::error::ErrorResponseBody {
                        error: Some(String::from("AuthenticationRequired")),
                        message: Some(String::from("Invalid identifier or password")),
                    },
                )?)?)
        }
    }

    impl XrpcClient for DummyXrpcClient {
        fn host(&self) -> &str {
            "http://localhost:8080"
        }
    }

    #[test]
    fn test_new() {
        let _ = AtpServiceClient::new(DummyXrpcClient);
    }

    #[tokio::test]
    async fn test_xrpc() {
        let client = AtpServiceClient::new(DummyXrpcClient);
        let result = client
            .service
            .com
            .atproto
            .server
            .create_session(crate::com::atproto::server::create_session::Input {
                identifier: String::from("test"),
                password: String::from("test"),
            })
            .await
            .expect_err("response should be error");
        match &result {
            atrium_xrpc::error::Error::XrpcResponse(xrpc_error) => {
                assert_eq!(xrpc_error.status, StatusCode::UNAUTHORIZED);
                assert_eq!(
                    xrpc_error.error,
                    Some(atrium_xrpc::error::XrpcErrorKind::Undefined(
                        atrium_xrpc::error::ErrorResponseBody {
                            error: Some(String::from("AuthenticationRequired")),
                            message: Some(String::from("Invalid identifier or password")),
                        }
                    ))
                );
            }
            _ => panic!("unexpected error type"),
        }
    }
}
