#![doc = include_str!("../README.md")]
pub mod client;
pub mod error;
pub use http;

use crate::error::{Error, XrpcError, XrpcErrorKind};
use async_trait::async_trait;
use http::{Method, Request, Response};
use serde::{de::DeserializeOwned, Serialize};

pub enum InputDataOrBytes<T>
where
    T: Serialize,
{
    Data(T),
    Bytes(Vec<u8>),
}

pub enum OutputDataOrBytes<T>
where
    T: DeserializeOwned,
{
    Data(T),
    Bytes(Vec<u8>),
}

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
        path: &str,
        parameters: Option<P>,
        input: Option<InputDataOrBytes<I>>,
        encoding: Option<String>,
    ) -> Result<OutputDataOrBytes<O>, self::Error<E>>
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
            match input {
                InputDataOrBytes::Data(data) => serde_json::to_vec(&data)?,
                InputDataOrBytes::Bytes(bytes) => bytes,
            }
        } else {
            Vec::new()
        };
        let (parts, body) = HttpClient::send(self, builder.body(body)?)
            .await
            .map_err(Error::HttpClient)?
            .into_parts();
        if parts.status.is_success() {
            if parts
                .headers
                .get(http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .map_or(false, |content_type| {
                    content_type.starts_with("application/json")
                })
            {
                Ok(OutputDataOrBytes::Data(serde_json::from_slice(&body)?))
            } else {
                Ok(OutputDataOrBytes::Bytes(body))
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
        json: bool,
        body: Vec<u8>,
    }

    #[async_trait]
    impl HttpClient for DummyClient {
        async fn send(
            &self,
            _req: Request<Vec<u8>>,
        ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
            let mut builder = Response::builder().status(self.status);
            if self.json {
                builder = builder.header(http::header::CONTENT_TYPE, "application/json")
            }
            Ok(builder.body(self.body.clone())?)
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
            let response = crate::XrpcClient::send::<_, (), _, _>(
                xrpc,
                http::Method::GET,
                "example",
                Some(params),
                None,
                None,
            )
            .await?;
            match response {
                crate::OutputDataOrBytes::Data(data) => Ok(data),
                _ => Err(crate::Error::UnexpectedResponseType),
            }
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        struct Parameters {}

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        struct Output {
            return_value: i32,
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
                json: true,
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
                json: true,
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
                _ => panic!("must be Error::XrpcResponse, got {error:?}"),
            }
        }

        #[tokio::test]
        async fn response_undefined_error() {
            let client = DummyClient {
                status: http::StatusCode::INTERNAL_SERVER_ERROR,
                json: true,
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
                _ => panic!("must be Error::XrpcResponse, got {error:?}"),
            };
        }
    }

    mod query {
        use super::*;

        mod bytes {
            use super::*;

            async fn get_bytes<T>(
                xrpc: &T,
                params: Parameters,
            ) -> Result<Vec<u8>, crate::Error<Error>>
            where
                T: crate::XrpcClient + Send + Sync,
            {
                let response = crate::XrpcClient::send::<_, (), (), _>(
                    xrpc,
                    http::Method::GET,
                    "example",
                    Some(params),
                    None,
                    None,
                )
                .await?;
                match response {
                    crate::OutputDataOrBytes::Bytes(bytes) => Ok(bytes),
                    _ => Err(crate::Error::UnexpectedResponseType),
                }
            }

            #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
            #[serde(rename_all = "camelCase")]
            struct Parameters {
                query: String,
            }

            #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
            #[serde(tag = "error", content = "message")]
            enum Error {}

            #[tokio::test]
            async fn response_ok() {
                let body = r"data".as_bytes().to_vec();
                let client = DummyClient {
                    status: http::StatusCode::OK,
                    json: false,
                    body: body.clone(),
                };
                let out = get_bytes(
                    &client,
                    Parameters {
                        query: "foo".into(),
                    },
                )
                .await
                .expect("must be ok");
                assert_eq!(out, body);
            }

            #[tokio::test]
            async fn response_unexpected() {
                let client = DummyClient {
                    status: http::StatusCode::OK,
                    json: true,
                    body: r"null".as_bytes().to_vec(),
                };
                let result = get_bytes(
                    &client,
                    Parameters {
                        query: "foo".into(),
                    },
                )
                .await;
                let error = result.expect_err("must be error");
                match &error {
                    crate::Error::UnexpectedResponseType => {}
                    _ => panic!("must be Error::UnexpectedResponseType, got {error:?}"),
                }
            }
        }
    }

    mod procedure {
        use super::*;

        mod no_content {
            use super::*;

            async fn create_data<T>(xrpc: &T, input: Input) -> Result<(), crate::Error<Error>>
            where
                T: crate::XrpcClient + Send + Sync,
            {
                let response = crate::XrpcClient::send::<(), _, (), _>(
                    xrpc,
                    http::Method::POST,
                    "example",
                    None,
                    Some(InputDataOrBytes::Data(input)),
                    None,
                )
                .await?;
                match response {
                    crate::OutputDataOrBytes::Bytes(_) => Ok(()),
                    _ => Err(crate::Error::UnexpectedResponseType),
                }
            }

            #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
            #[serde(rename_all = "camelCase")]
            struct Input {
                value: i32,
            }

            #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
            #[serde(tag = "error", content = "message")]
            enum Error {}

            #[tokio::test]
            async fn response_ok() {
                let client = DummyClient {
                    status: http::StatusCode::OK,
                    json: false,
                    body: Vec::new(),
                };
                create_data(&client, Input { value: 42 })
                    .await
                    .expect("must be ok");
            }

            #[tokio::test]
            async fn response_unexpected() {
                let client = DummyClient {
                    status: http::StatusCode::OK,
                    json: true,
                    body: r"null".as_bytes().to_vec(),
                };
                let result = create_data(&client, Input { value: 42 }).await;
                let error = result.expect_err("must be error");
                match &error {
                    crate::Error::UnexpectedResponseType => {}
                    _ => panic!("must be Error::UnexpectedResponseType, got {error:?}"),
                }
            }
        }

        mod bytes {
            use super::*;

            async fn create_data<T>(xrpc: &T, input: Vec<u8>) -> Result<Output, crate::Error<Error>>
            where
                T: crate::XrpcClient + Send + Sync,
            {
                let response = crate::XrpcClient::send::<(), Vec<u8>, _, _>(
                    xrpc,
                    http::Method::POST,
                    "example",
                    None,
                    Some(InputDataOrBytes::Bytes(input)),
                    None,
                )
                .await?;
                match response {
                    crate::OutputDataOrBytes::Data(data) => Ok(data),
                    _ => Err(crate::Error::UnexpectedResponseType),
                }
            }

            #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
            #[serde(rename_all = "camelCase")]
            struct Output {
                return_value: i32,
            }

            #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
            #[serde(tag = "error", content = "message")]
            enum Error {}

            #[tokio::test]
            async fn response_ok() {
                let client = DummyClient {
                    status: http::StatusCode::OK,
                    json: true,
                    body: r#"{"returnValue":42}"#.as_bytes().to_vec(),
                };
                create_data(&client, "data".as_bytes().to_vec())
                    .await
                    .expect("must be ok");
            }

            #[tokio::test]
            async fn response_unexpected() {
                let client = DummyClient {
                    status: http::StatusCode::OK,
                    json: false,
                    body: r#"{"returnValue":42}"#.as_bytes().to_vec(),
                };
                let result = create_data(&client, "data".as_bytes().to_vec()).await;
                let error = result.expect_err("must be error");
                match &error {
                    crate::Error::UnexpectedResponseType => {}
                    _ => panic!("must be Error::UnexpectedResponseType, got {error:?}"),
                }
            }
        }
    }
}
