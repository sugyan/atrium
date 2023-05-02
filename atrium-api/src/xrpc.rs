use async_trait::async_trait;
use http::{header, Method, Request, Response};
use serde::de::DeserializeOwned;
use std::error::Error;
use std::fmt::Debug;

/// [Custom error codes and descriptions](https://atproto.com/specs/xrpc#custom-error-codes-and-descriptions)
///
/// ```typescript
/// export const errorResponseBody = z.object({
///   error: z.string().optional(),
///   message: z.string().optional(),
/// })
/// export type ErrorResponseBody = z.infer<typeof errorResponseBody>
/// ```
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ErrorResponseBody {
    pub error: Option<String>,
    pub message: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum XrpcError<E>
where
    E: Debug,
{
    Custom(E),
    Undefined(ErrorResponseBody),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XrpcResponseError<E>
where
    E: Debug + PartialEq + Eq,
{
    pub status: http::StatusCode,
    pub error: Option<XrpcError<E>>,
}

impl<E> std::fmt::Display for XrpcResponseError<E>
where
    E: Debug + PartialEq + Eq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("XrpcResponseError({})", self.status))?;
        if let Some(error) = &self.error {
            f.write_str(": ")?;
            match error {
                XrpcError::Custom(err) => {
                    err.fmt(f)?;
                }
                XrpcError::Undefined(err) => {
                    if let Some(e) = &err.error {
                        f.write_fmt(format_args!("`{}` {:?}", e, err.message))?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl<E> Error for XrpcResponseError<E> where E: Debug + PartialEq + Eq {}

#[async_trait]
pub trait HttpClient {
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Box<dyn Error>>;
}

#[async_trait]
pub trait XrpcClient: HttpClient {
    fn host(&self) -> &str;
    fn auth(&self) -> Option<&str>;
    async fn send<E>(
        &self,
        method: Method,
        path: &str,
        query: Option<String>,
        input: Option<Vec<u8>>,
        encoding: Option<String>,
    ) -> Result<Vec<u8>, Box<dyn Error>>
    where
        E: Debug + DeserializeOwned + PartialEq + Eq + 'static,
    {
        let mut uri = format!("{}/xrpc/{path}", self.host());
        if let Some(query) = &query {
            uri += "?";
            uri += query;
        };
        println!("{} {}", method, uri);
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(encoding) = encoding {
            builder = builder.header(header::CONTENT_TYPE, encoding);
        }
        if let Some(token) = self.auth() {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        let (parts, body) = HttpClient::send(self, builder.body(input.unwrap_or_default())?)
            .await?
            .into_parts();
        if parts.status.is_success() {
            Ok(body)
        } else {
            Err(Box::new(XrpcResponseError {
                status: parts.status,
                error: serde_json::from_slice::<XrpcError<E>>(&body).ok(),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    mod example {
        #[async_trait::async_trait]
        pub trait GetExample: crate::xrpc::XrpcClient {
            async fn get_example(
                &self,
                params: Parameters,
            ) -> Result<Output, Box<dyn std::error::Error>> {
                let body = crate::xrpc::XrpcClient::send::<Error>(
                    self,
                    http::Method::GET,
                    "example",
                    Some(serde_urlencoded::to_string(&params)?),
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
        async fn send(&self, _req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Box<dyn Error>> {
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
        fn auth(&self) -> Option<&str> {
            None
        }
    }

    impl example::GetExample for DummyClient {}

    #[test]
    fn deserialize_xrpc_error() {
        {
            let body = r#"{"error":"InvalidToken","message":"Invalid token"}"#;
            let err = serde_json::from_str::<XrpcError<_>>(body).expect("deserialize");
            assert_eq!(
                err,
                XrpcError::Custom(example::Error::InvalidToken(Some(String::from(
                    "Invalid token"
                ))))
            );
        }
        {
            let body = r#"{"error":"ExpiredToken"}"#;
            let err = serde_json::from_str::<XrpcError<_>>(body).expect("deserialize");
            assert_eq!(err, XrpcError::Custom(example::Error::ExpiredToken(None)));
        }
        {
            let body = r#"{"error":"Unknown","message":"Something wrong"}"#;
            let err = serde_json::from_str::<XrpcError<example::Error>>(body).expect("deserialize");
            assert_eq!(
                err,
                XrpcError::Undefined(ErrorResponseBody {
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
        let error = client
            .get_example(example::Parameters {})
            .await
            .expect_err("must be error");
        assert_eq!(
            error.downcast_ref::<XrpcResponseError<example::Error>>(),
            Some(&XrpcResponseError {
                status: http::StatusCode::BAD_REQUEST,
                error: Some(XrpcError::Custom(example::Error::InvalidToken(Some(
                    String::from("Message")
                ))))
            })
        );
        assert_eq!(
            error.to_string(),
            r#"XrpcResponseError(400 Bad Request): InvalidToken(Some("Message"))"#
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
        let error = client
            .get_example(example::Parameters {})
            .await
            .expect_err("must be error");
        assert_eq!(
            error.downcast_ref::<XrpcResponseError<example::Error>>(),
            Some(&XrpcResponseError {
                status: http::StatusCode::INTERNAL_SERVER_ERROR,
                error: Some(XrpcError::Undefined(ErrorResponseBody {
                    error: Some(String::from("Unknown")),
                    message: Some(String::from("Something wrong"))
                }))
            })
        );
        assert_eq!(
            error.to_string(),
            r#"XrpcResponseError(500 Internal Server Error): `Unknown` Some("Something wrong")"#
        );
    }
}
