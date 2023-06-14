#![doc = include_str!("../README.md")]
pub mod client;
pub mod error;
pub use http;

use crate::error::{Error, XrpcError, XrpcErrorKind};
use async_trait::async_trait;
use http::{Method, Request, Response};
use serde::{de::DeserializeOwned, Serialize};

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
    fn auth(&self, is_refresh: bool) -> Option<String> {
        None
    }
    async fn send<P, I, O, E>(
        &self,
        method: Method,
        path: String,
        parameters: Option<P>,
        input: Option<I>,
        encoding: Option<String>,
    ) -> Result<O, self::Error<E>>
    where
        P: Serialize + Send,
        I: Serialize + Send,
        O: DeserializeOwned,
        E: DeserializeOwned,
    {
        let mut uri = format!("{}/xrpc/{path}", self.host());
        if let Some(p) = &parameters {
            serde_qs::to_string(p).map(|qs| {
                uri += "?";
                uri += &qs;
            })?;
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
        let body = if let Some(input) = input {
            serde_json::to_vec(&input)?
        } else {
            Vec::new()
        };
        let (parts, body) = HttpClient::send(self, builder.body(body)?)
            .await
            .map_err(Error::HttpClient)?
            .into_parts();
        if parts.status.is_success() {
            if body.is_empty() {
                // An empty response body can not be deserialized,
                // but then the Output schema should be defined as a unit structure.
                // so `"null"` is used to match the structure.
                Ok(serde_json::from_str("null")?)
            } else {
                Ok(serde_json::from_slice(&body)?)
            }
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
    use super::*;

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

    mod errors {
        use super::*;

        async fn get_example<T>(xrpc: &T, params: Parameters) -> Result<Output, crate::Error<Error>>
        where
            T: crate::XrpcClient + Send + Sync,
        {
            crate::XrpcClient::send::<_, Input, _, _>(
                xrpc,
                http::Method::GET,
                String::from("example"),
                Some(params),
                None,
                None,
            )
            .await
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        struct Parameters {}

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        struct Input;

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        struct Output {
            pub return_value: i32,
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
        #[serde(tag = "error", content = "message")]
        enum Error {
            InvalidToken(Option<String>),
            ExpiredToken(Option<String>),
        }

        #[test]
        fn deserialize_xrpc_error() {
            {
                let body = r#"{"error":"InvalidToken","message":"Invalid token"}"#;
                let err = serde_json::from_str::<XrpcErrorKind<_>>(body).expect("deserialize");
                assert_eq!(
                    err,
                    XrpcErrorKind::Custom(Error::InvalidToken(Some(String::from("Invalid token"))))
                );
            }
            {
                let body = r#"{"error":"ExpiredToken"}"#;
                let err = serde_json::from_str::<XrpcErrorKind<_>>(body).expect("deserialize");
                assert_eq!(err, XrpcErrorKind::Custom(Error::ExpiredToken(None)));
            }
            {
                let body = r#"{"error":"Unknown","message":"Something wrong"}"#;
                let err = serde_json::from_str::<XrpcErrorKind<Error>>(body).expect("deserialize");
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
            let out = get_example(&client, Parameters {})
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
            let result = get_example(&client, Parameters {}).await;
            let error = result.expect_err("must be error");
            match &error {
                crate::Error::XrpcResponse(err) => {
                    assert_eq!(
                        err,
                        &XrpcError {
                            status: http::StatusCode::BAD_REQUEST,
                            error: Some(XrpcErrorKind::Custom(Error::InvalidToken(Some(
                                String::from("Message")
                            ))))
                        }
                    );
                }
                _ => panic!("must be Error::XrpcResponse"),
            }
        }

        #[tokio::test]
        async fn response_undefined_error() {
            let client = DummyClient {
                status: http::StatusCode::INTERNAL_SERVER_ERROR,
                body: r#"{"error":"Unknown","message":"Something wrong"}"#
                    .as_bytes()
                    .to_vec(),
            };
            let result = get_example(&client, Parameters {}).await;
            let error = result.expect_err("must be error");
            match &error {
                crate::Error::XrpcResponse(err) => {
                    assert_eq!(
                        err,
                        &XrpcError {
                            status: http::StatusCode::INTERNAL_SERVER_ERROR,
                            error: Some(XrpcErrorKind::Undefined(
                                crate::error::ErrorResponseBody {
                                    error: Some(String::from("Unknown")),
                                    message: Some(String::from("Something wrong"))
                                }
                            ))
                        }
                    );
                }
                _ => panic!("must be Error::XrpcResponse"),
            };
        }
    }
}
