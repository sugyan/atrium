#![doc = include_str!("../README.md")]
pub mod error;

use crate::error::{Error, XrpcError, XrpcErrorKind};
use async_trait::async_trait;
use http::{Method, Request, Response};
use serde::{de::DeserializeOwned, Serialize};

/// A type which can be used as a parameter of [`XrpcRequest`].
///
/// JSON serializable data or raw bytes.
pub enum InputDataOrBytes<T>
where
    T: Serialize,
{
    Data(T),
    Bytes(Vec<u8>),
}

/// A type which can be used as a return value of [`XrpcClient::send_xrpc()`].
///
/// JSON deserializable data or raw bytes.
pub enum OutputDataOrBytes<T>
where
    T: DeserializeOwned,
{
    Data(T),
    Bytes(Vec<u8>),
}

/// An abstract HTTP client.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait HttpClient {
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>>;
}

/// A request which can be executed with [`XrpcClient::send_xrpc()`].
pub struct XrpcRequest<P, I>
where
    I: Serialize,
{
    pub method: Method,
    pub path: String,
    pub parameters: Option<P>,
    pub input: Option<InputDataOrBytes<I>>,
    pub encoding: Option<String>,
}

pub type XrpcResult<O, E> = Result<OutputDataOrBytes<O>, self::Error<E>>;

/// An abstract XRPC client.
///
/// [`send_xrpc()`](XrpcClient::send_xrpc) method has a default implementation,
/// which wraps the [`HttpClient::send_http()`]` method to handle input and output as an XRPC Request.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait XrpcClient: HttpClient {
    fn base_uri(&self) -> String;
    #[allow(unused_variables)]
    async fn auth(&self, is_refresh: bool) -> Option<String> {
        None
    }
    async fn send_xrpc<P, I, O, E>(&self, request: &XrpcRequest<P, I>) -> XrpcResult<O, E>
    where
        P: Serialize + Send + Sync,
        I: Serialize + Send + Sync,
        O: DeserializeOwned + Send + Sync,
        E: DeserializeOwned + Send + Sync,
    {
        let mut uri = format!("{}/xrpc/{}", self.base_uri(), request.path);
        if let Some(p) = &request.parameters {
            serde_html_form::to_string(p).map(|qs| {
                uri += "?";
                uri += &qs;
            })?;
        };
        let mut builder = Request::builder().method(&request.method).uri(&uri);
        if let Some(encoding) = &request.encoding {
            builder = builder.header(http::header::CONTENT_TYPE, encoding);
        }
        if let Some(token) = self
            .auth(
                request.method == Method::POST
                    && request.path == "com.atproto.server.refreshSession",
            )
            .await
        {
            builder = builder.header(http::header::AUTHORIZATION, format!("Bearer {}", token));
        }
        let body = if let Some(input) = &request.input {
            match input {
                InputDataOrBytes::Data(data) => serde_json::to_vec(&data)?,
                InputDataOrBytes::Bytes(bytes) => bytes.clone(),
            }
        } else {
            Vec::new()
        };
        let (parts, body) = self
            .send_http(builder.body(body)?)
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
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    struct DummyClient {
        status: http::StatusCode,
        json: bool,
        body: Vec<u8>,
    }

    #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl HttpClient for DummyClient {
        async fn send_http(
            &self,
            _request: Request<Vec<u8>>,
        ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
            let mut builder = Response::builder().status(self.status);
            if self.json {
                builder = builder.header(http::header::CONTENT_TYPE, "application/json")
            }
            Ok(builder.body(self.body.clone())?)
        }
    }

    impl XrpcClient for DummyClient {
        fn base_uri(&self) -> String {
            "https://example.com".into()
        }
    }

    mod errors {
        use super::*;

        async fn get_example<T>(xrpc: &T, params: Parameters) -> Result<Output, crate::Error<Error>>
        where
            T: crate::XrpcClient + Send + Sync,
        {
            let response = xrpc
                .send_xrpc::<_, (), _, _>(&XrpcRequest {
                    method: http::Method::GET,
                    path: "example".into(),
                    parameters: Some(params),
                    input: None,
                    encoding: None,
                })
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
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
                let response = xrpc
                    .send_xrpc::<_, (), (), _>(&XrpcRequest {
                        method: http::Method::GET,
                        path: "example".into(),
                        parameters: Some(params),
                        input: None,
                        encoding: None,
                    })
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
            #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
            #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
                let response = xrpc
                    .send_xrpc::<(), _, (), _>(&XrpcRequest {
                        method: http::Method::POST,
                        path: "example".into(),
                        parameters: None,
                        input: Some(InputDataOrBytes::Data(input)),
                        encoding: None,
                    })
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
            #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
            #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
                let response = xrpc
                    .send_xrpc::<(), Vec<u8>, _, _>(&XrpcRequest {
                        method: http::Method::POST,
                        path: "example".into(),
                        parameters: None,
                        input: Some(InputDataOrBytes::Bytes(input)),
                        encoding: None,
                    })
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
            #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
            #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
