use atrium_xrpc::{InputDataOrBytes, OutputDataOrBytes, XrpcClient, XrpcRequest};
use futures::future::join_all;
use http::Method;
use mockito::{Matcher, Server};
use serde::{Deserialize, Serialize};
use tokio::task::JoinError;

#[derive(Serialize, Deserialize, Debug)]
struct Parameters {
    query: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Input {
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Output {
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "error", content = "message")]
enum Error {
    Bad,
}

async fn run_query(
    client: impl XrpcClient + Send + Sync,
    path: String,
) -> Result<Output, atrium_xrpc::error::Error<Error>> {
    let response = client
        .send_xrpc::<_, (), _, _>(&XrpcRequest {
            method: Method::GET,
            path,
            parameters: Some(Parameters {
                query: "foo".into(),
            }),
            input: None,
            encoding: None,
        })
        .await?;
    match response {
        OutputDataOrBytes::Bytes(_) => Err(atrium_xrpc::error::Error::UnexpectedResponseType),
        OutputDataOrBytes::Data(out) => Ok(out),
    }
}

async fn run_procedure(
    client: impl XrpcClient + Send + Sync,
    path: String,
) -> Result<Output, atrium_xrpc::error::Error<Error>> {
    let response = client
        .send_xrpc::<(), _, _, _>(&XrpcRequest {
            method: Method::POST,
            path,
            parameters: None,
            input: Some(InputDataOrBytes::Data(Input { data: "foo".into() })),
            encoding: Some("application/json".into()),
        })
        .await?;
    match response {
        OutputDataOrBytes::Bytes(_) => Err(atrium_xrpc::error::Error::UnexpectedResponseType),
        OutputDataOrBytes::Data(out) => Ok(out),
    }
}

#[tokio::test]
async fn send_query() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let mock_ok = server
        .mock("GET", "/xrpc/test/ok")
        .match_query(Matcher::UrlEncoded("query".into(), "foo".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"data": "bar"}"#)
        .create_async()
        .await;
    let mock_err = server
        .mock("GET", "/xrpc/test/err")
        .match_query(Matcher::UrlEncoded("query".into(), "foo".into()))
        .with_status(400)
        .with_body(r#"{"error": "Bad"}"#)
        .create_async()
        .await;
    let mock_server_error = server
        .mock("GET", "/xrpc/test/500")
        .match_query(Matcher::Any)
        .with_status(500)
        .create_async()
        .await;

    async fn run(
        base_uri: &str,
        path: &str,
    ) -> Vec<Result<Result<Output, atrium_xrpc::error::Error<Error>>, JoinError>> {
        let handles = vec![
            #[cfg(feature = "isahc")]
            tokio::spawn(run_query(
                crate::isahc::IsahcClientBuilder::new(base_uri)
                    .client(
                        isahc::HttpClient::builder()
                            .build()
                            .expect("client should be successfully built"),
                    )
                    .build(),
                path.to_string(),
            )),
            #[cfg(feature = "reqwest-native")]
            tokio::spawn(run_query(
                crate::reqwest::ReqwestClientBuilder::new(base_uri)
                    .client(
                        reqwest::ClientBuilder::new()
                            .use_native_tls()
                            .build()
                            .expect("client should be successfully built"),
                    )
                    .build(),
                path.to_string(),
            )),
            #[cfg(feature = "reqwest-rustls")]
            tokio::spawn(run_query(
                crate::reqwest::ReqwestClientBuilder::new(base_uri)
                    .client(
                        reqwest::ClientBuilder::new()
                            .use_rustls_tls()
                            .build()
                            .expect("client should be successfully built"),
                    )
                    .build(),
                path.to_string(),
            )),
            #[cfg(feature = "surf")]
            tokio::spawn(run_query(
                crate::surf::SurfClient::new(
                    base_uri,
                    surf::Client::with_http_client(http_client::h1::H1Client::new()),
                ),
                path.to_string(),
            )),
        ];
        join_all(handles).await
    }

    // Ok
    {
        let results = run(&server.url(), "test/ok").await;
        let len = results.len();
        for result in results {
            let output = result?.expect("xrpc response should be ok");
            assert_eq!(output.data, "bar");
        }
        mock_ok.expect(len).assert_async().await;
    }
    // Err (XrpcError)
    {
        let results = run(&server.url(), "test/err").await;
        let len = results.len();
        for result in results {
            let err = result?.expect_err("xrpc response should be error");
            if let atrium_xrpc::error::Error::XrpcResponse(e) = err {
                assert_eq!(e.status, 400);
                if let Some(atrium_xrpc::error::XrpcErrorKind::Custom(Error::Bad)) = e.error {
                } else {
                    panic!("unexpected error kind: {e:?}");
                }
            } else {
                panic!("unexpected error: {err:?}");
            }
        }
        mock_err.expect(len).assert_async().await;
    }
    // Err (server error)
    {
        let results = run(&server.url(), "test/500").await;
        let len = results.len();
        for result in results {
            let err = result?.expect_err("xrpc response should be error");
            if let atrium_xrpc::error::Error::XrpcResponse(e) = err {
                assert_eq!(e.status, 500);
                assert!(e.error.is_none());
            } else {
                panic!("unexpected error: {err:?}");
            }
        }
        mock_server_error.expect(len).assert_async().await;
    }
    Ok(())
}

#[tokio::test]
async fn send_procedure() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let mock_ok = server
        .mock("POST", "/xrpc/test/ok")
        .match_header("content-type", "application/json")
        .match_body(Matcher::JsonString(r#"{"data": "foo"}"#.into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"data": "bar"}"#)
        .create_async()
        .await;
    let mock_err = server
        .mock("POST", "/xrpc/test/err")
        .match_header("content-type", "application/json")
        .match_body(Matcher::JsonString(r#"{"data": "foo"}"#.into()))
        .with_status(400)
        .with_body(r#"{"error": "Bad"}"#)
        .create_async()
        .await;
    let mock_server_error = server
        .mock("POST", "/xrpc/test/500")
        .match_query(Matcher::Any)
        .with_status(500)
        .create_async()
        .await;

    async fn run(
        base_uri: &str,
        path: &str,
    ) -> Vec<Result<Result<Output, atrium_xrpc::error::Error<Error>>, JoinError>> {
        let handles = vec![
            #[cfg(feature = "isahc")]
            tokio::spawn(run_procedure(
                crate::isahc::IsahcClientBuilder::new(base_uri)
                    .client(
                        isahc::HttpClient::builder()
                            .build()
                            .expect("client should be successfully built"),
                    )
                    .build(),
                path.to_string(),
            )),
            #[cfg(feature = "reqwest-native")]
            tokio::spawn(run_procedure(
                crate::reqwest::ReqwestClientBuilder::new(base_uri)
                    .client(
                        reqwest::ClientBuilder::new()
                            .use_native_tls()
                            .build()
                            .expect("client should be successfully built"),
                    )
                    .build(),
                path.to_string(),
            )),
            #[cfg(feature = "reqwest-rustls")]
            tokio::spawn(run_procedure(
                crate::reqwest::ReqwestClientBuilder::new(base_uri)
                    .client(
                        reqwest::ClientBuilder::new()
                            .use_rustls_tls()
                            .build()
                            .expect("client should be successfully built"),
                    )
                    .build(),
                path.to_string(),
            )),
            #[cfg(feature = "surf")]
            tokio::spawn(run_procedure(
                crate::surf::SurfClient::new(
                    base_uri,
                    surf::Client::with_http_client(http_client::h1::H1Client::new()),
                ),
                path.to_string(),
            )),
        ];
        join_all(handles).await
    }

    // Ok
    {
        let results = run(&server.url(), "test/ok").await;
        let len = results.len();
        for result in results {
            let output = result?.expect("xrpc response should be ok");
            assert_eq!(output.data, "bar");
        }
        mock_ok.expect(len).assert_async().await;
    }
    // Err (XrpcError)
    {
        let results = run(&server.url(), "test/err").await;
        let len = results.len();
        for result in results {
            let err = result?.expect_err("xrpc response should be error");
            if let atrium_xrpc::error::Error::XrpcResponse(e) = err {
                assert_eq!(e.status, 400);
                if let Some(atrium_xrpc::error::XrpcErrorKind::Custom(Error::Bad)) = e.error {
                } else {
                    panic!("unexpected error kind: {e:?}");
                }
            } else {
                panic!("unexpected error: {err:?}");
            }
        }
        mock_err.expect(len).assert_async().await;
    }
    // Err (server error)
    {
        let results = run(&server.url(), "test/500").await;
        let len = results.len();
        for result in results {
            let err = result?.expect_err("xrpc response should be error");
            if let atrium_xrpc::error::Error::XrpcResponse(e) = err {
                assert_eq!(e.status, 500);
                assert!(e.error.is_none());
            } else {
                panic!("unexpected error: {err:?}");
            }
        }
        mock_server_error.expect(len).assert_async().await;
    }
    Ok(())
}
