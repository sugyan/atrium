#![doc = include_str!("../README.md")]
pub mod error;
pub mod reqwest;
pub use http;

use crate::error::{Error, XrpcError, XrpcErrorKind};
use async_trait::async_trait;
use http::{Method, Request, Response};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[async_trait]
pub trait HttpClient {
    async fn send(
        &self,
        req: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>>;
}

#[async_trait]
pub trait XrpcClient: HttpClient {
    fn host(&self) -> &str;
    #[allow(unused_variables)]
    fn auth(&self, is_refresh: bool) -> Option<&str> {
        None
    }
    async fn send<E>(
        &self,
        method: Method,
        path: &str,
        query: Option<String>,
        input: Option<Vec<u8>>,
        encoding: Option<String>,
    ) -> Result<Vec<u8>, self::Error<E>>
    where
        E: DeserializeOwned + Debug,
    {
        let mut uri = format!("{}/xrpc/{path}", self.host());
        if let Some(query) = &query {
            uri += "?";
            uri += query;
        };
        let mut builder = Request::builder().method(&method).uri(&uri);
        if let Some(encoding) = encoding {
            builder = builder.header(http::header::CONTENT_TYPE, encoding);
        }
        if let Some(token) =
            self.auth(method == Method::POST && path == "com.atproto.server.refreshSession")
        {
            builder = builder.header(http::header::AUTHORIZATION, format!("Bearer {}", token));
        }
        let (parts, body) = HttpClient::send(self, builder.body(input.unwrap_or_default())?)
            .await
            .map_err(Error::HttpClient)?
            .into_parts();
        if parts.status.is_success() {
            Ok(body)
        } else {
            Err(Error::XrpcResponse(XrpcError {
                status: parts.status,
                error: serde_json::from_slice::<XrpcErrorKind<E>>(&body).ok(),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    mod example {
        #[async_trait::async_trait]
        pub trait GetExample: crate::XrpcClient {
            async fn get_example(&self, params: Parameters) -> Result<Output, crate::Error<Error>> {
                let body = crate::XrpcClient::send::<Error>(
                    self,
                    http::Method::GET,
                    "example",
                    Some(serde_qs::to_string(&params)?),
                    None,
                    None,
                )
                .await?;
                serde_json::from_slice(&body).map_err(|e| e.into())
            }
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct Parameters {}

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct Output {
            pub return_value: i32,
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
        #[serde(tag = "error", content = "message")]
        pub enum Error {
            InvalidToken(Option<String>),
            ExpiredToken(Option<String>),
        }
    }

    use super::*;
    use example::GetExample;

    struct DummyClient {
        status: http::StatusCode,
        body: Vec<u8>,
    }

    #[async_trait]
    impl HttpClient for DummyClient {
        async fn send(
            &self,
            _req: Request<Vec<u8>>,
        ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
            Response::builder()
                .status(self.status)
                .body(self.body.clone())
                .map_err(|e| e.into())
        }
    }

    #[async_trait]
    impl XrpcClient for DummyClient {
        fn host(&self) -> &str {
            "https://example.com"
        }
    }

    impl example::GetExample for DummyClient {}

    #[test]
    fn deserialize_xrpc_error() {
        {
            let body = r#"{"error":"InvalidToken","message":"Invalid token"}"#;
            let err = serde_json::from_str::<XrpcErrorKind<_>>(body).expect("deserialize");
            assert_eq!(
                err,
                XrpcErrorKind::Custom(example::Error::InvalidToken(Some(String::from(
                    "Invalid token"
                ))))
            );
        }
        {
            let body = r#"{"error":"ExpiredToken"}"#;
            let err = serde_json::from_str::<XrpcErrorKind<_>>(body).expect("deserialize");
            assert_eq!(
                err,
                XrpcErrorKind::Custom(example::Error::ExpiredToken(None))
            );
        }
        {
            let body = r#"{"error":"Unknown","message":"Something wrong"}"#;
            let err =
                serde_json::from_str::<XrpcErrorKind<example::Error>>(body).expect("deserialize");
            assert_eq!(
                err,
                XrpcErrorKind::Undefined(crate::error::ErrorResponseBody {
                    error: Some(String::from("Unknown")),
                    message: Some(String::from("Something wrong")),
                })
            );
        }
    }

    #[tokio::test]
    async fn response_ok() {
        let client = DummyClient {
            status: http::StatusCode::OK,
            body: r#"{"returnValue":42}"#.as_bytes().to_vec(),
        };
        let out = client
            .get_example(example::Parameters {})
            .await
            .expect("must be ok");
        assert_eq!(out.return_value, 42);
    }

    #[tokio::test]
    async fn response_custom_error() {
        let client = DummyClient {
            status: http::StatusCode::BAD_REQUEST,
            body: r#"{"error":"InvalidToken","message":"Message"}"#.as_bytes().to_vec(),
        };
        let result = client.get_example(example::Parameters {}).await;
        let error = result.expect_err("must be error");
        match &error {
            Error::XrpcResponse(err) => {
                assert_eq!(
                    err,
                    &XrpcError {
                        status: http::StatusCode::BAD_REQUEST,
                        error: Some(XrpcErrorKind::Custom(example::Error::InvalidToken(Some(
                            String::from("Message")
                        ))))
                    }
                );
            }
            _ => panic!("must be Error::XrpcResponse"),
        }
        assert_eq!(
            error.to_string(),
            r#"XRPC response error: 400 Bad Request (InvalidToken(Some("Message")))"#
        );
    }

    #[tokio::test]
    async fn response_undefined_error() {
        let client = DummyClient {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            body: r#"{"error":"Unknown","message":"Something wrong"}"#
                .as_bytes()
                .to_vec(),
        };
        let result = client.get_example(example::Parameters {}).await;
        let error = result.expect_err("must be error");
        match &error {
            Error::XrpcResponse(err) => {
                assert_eq!(
                    err,
                    &XrpcError {
                        status: http::StatusCode::INTERNAL_SERVER_ERROR,
                        error: Some(XrpcErrorKind::Undefined(crate::error::ErrorResponseBody {
                            error: Some(String::from("Unknown")),
                            message: Some(String::from("Something wrong"))
                        }))
                    }
                );
            }
            _ => panic!("must be Error::XrpcResponse"),
        };
        assert_eq!(
            error.to_string(),
            r#"XRPC response error: 500 Internal Server Error (`Unknown` Some("Something wrong"))"#
        );
    }
}
