#![doc = include_str!("../README.md")]
pub mod error;
mod traits;
pub mod types;

pub use crate::error::{Error, Result};
pub use crate::traits::{HttpClient, XrpcClient};
pub use crate::types::{InputDataOrBytes, OutputDataOrBytes, XrpcRequest};
pub use http;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{XrpcError, XrpcErrorKind};
    use crate::{HttpClient, XrpcClient};
    use async_trait::async_trait;
    use http::{Request, Response};
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
        ) -> core::result::Result<
            Response<Vec<u8>>,
            Box<dyn std::error::Error + Send + Sync + 'static>,
        > {
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

        async fn get_example<T>(xrpc: &T, params: Parameters) -> Result<Output, Error>
        where
            T: crate::XrpcClient + Send + Sync,
        {
            let response = xrpc
                .send_xrpc::<_, (), _, _>(&XrpcRequest {
                    method: http::Method::GET,
                    nsid: "example".into(),
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
            let out = get_example(&client, Parameters {}).await.expect("must be ok");
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
                body: r#"{"error":"Unknown","message":"Something wrong"}"#.as_bytes().to_vec(),
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

            async fn get_bytes<T>(xrpc: &T, params: Parameters) -> Result<Vec<u8>, Error>
            where
                T: crate::XrpcClient + Send + Sync,
            {
                let response = xrpc
                    .send_xrpc::<_, (), (), _>(&XrpcRequest {
                        method: http::Method::GET,
                        nsid: "example".into(),
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
                let client =
                    DummyClient { status: http::StatusCode::OK, json: false, body: body.clone() };
                let out = get_bytes(&client, Parameters { query: "foo".into() })
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
                let result = get_bytes(&client, Parameters { query: "foo".into() }).await;
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

            async fn create_data<T>(xrpc: &T, input: Input) -> Result<(), Error>
            where
                T: crate::XrpcClient + Send + Sync,
            {
                let response = xrpc
                    .send_xrpc::<(), _, (), _>(&XrpcRequest {
                        method: http::Method::POST,
                        nsid: "example".into(),
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
                let client =
                    DummyClient { status: http::StatusCode::OK, json: false, body: Vec::new() };
                create_data(&client, Input { value: 42 }).await.expect("must be ok");
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

            async fn create_data<T>(xrpc: &T, input: Vec<u8>) -> Result<Output, Error>
            where
                T: crate::XrpcClient + Send + Sync,
            {
                let response = xrpc
                    .send_xrpc::<(), Vec<u8>, _, _>(&XrpcRequest {
                        method: http::Method::POST,
                        nsid: "example".into(),
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
                create_data(&client, "data".as_bytes().to_vec()).await.expect("must be ok");
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
