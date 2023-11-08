use atrium_xrpc::{OutputDataOrBytes, XrpcClient, XrpcRequest};
use futures::future::join_all;
use http::Method;
use mockito::{Matcher, Server};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Parameters {
    query: String,
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

#[tokio::test]
async fn test_query() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let mock_ok = server
        .mock("GET", "/xrpc/test/ok")
        .match_query(Matcher::UrlEncoded("query".into(), "bar".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"data": "foo"}"#)
        .create_async()
        .await;
    let mock_err = server
        .mock("GET", "/xrpc/test/err")
        .match_query(Matcher::UrlEncoded("query".into(), "bar".into()))
        .with_status(400)
        .with_body(r#"{"error": "Bad"}"#)
        .create_async()
        .await;

    async fn run(
        client: impl XrpcClient + Send + Sync,
        ok: bool,
    ) -> Result<Output, atrium_xrpc::error::Error<Error>> {
        let response = client
            .send_xrpc::<_, (), _, _>(&XrpcRequest {
                method: Method::GET,
                path: (if ok { "test/ok" } else { "test/err" }).into(),
                parameters: Some(Parameters {
                    query: "bar".into(),
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
    {
        let handles = vec![
            #[cfg(feature = "isahc")]
            tokio::spawn(run(
                crate::isahc::IsahcClientBuilder::new(server.url())
                    .client(isahc::HttpClient::builder().build()?)
                    .build(),
                true,
            )),
            #[cfg(feature = "reqwest-native")]
            tokio::spawn(run(
                crate::reqwest::ReqwestClientBuilder::new(server.url())
                    .client(reqwest::ClientBuilder::new().use_native_tls().build()?)
                    .build(),
                true,
            )),
            #[cfg(feature = "reqwest-rustls")]
            tokio::spawn(run(
                crate::reqwest::ReqwestClientBuilder::new(server.url())
                    .client(reqwest::ClientBuilder::new().use_rustls_tls().build()?)
                    .build(),
                true,
            )),
            #[cfg(feature = "surf")]
            tokio::spawn(run(
                crate::surf::SurfClient::new(
                    server.url(),
                    surf::Client::with_http_client(http_client::h1::H1Client::new()),
                ),
                true,
            )),
        ];
        let len = handles.len();
        for result in join_all(handles).await {
            let output = result?.expect("xrpc response should be ok");
            assert_eq!(output.data, "foo");
        }
        mock_ok.expect(len).assert_async().await;
    }
    {
        let handles = vec![
            #[cfg(feature = "isahc")]
            tokio::spawn(run(
                crate::isahc::IsahcClientBuilder::new(server.url())
                    .client(isahc::HttpClient::builder().build()?)
                    .build(),
                false,
            )),
            #[cfg(feature = "reqwest-native")]
            tokio::spawn(run(
                crate::reqwest::ReqwestClientBuilder::new(server.url())
                    .client(reqwest::ClientBuilder::new().use_native_tls().build()?)
                    .build(),
                false,
            )),
            #[cfg(feature = "reqwest-rustls")]
            tokio::spawn(run(
                crate::reqwest::ReqwestClientBuilder::new(server.url())
                    .client(reqwest::ClientBuilder::new().use_rustls_tls().build()?)
                    .build(),
                false,
            )),
            #[cfg(feature = "surf")]
            tokio::spawn(run(
                crate::surf::SurfClient::new(
                    server.url(),
                    surf::Client::with_http_client(http_client::h1::H1Client::new()),
                ),
                false,
            )),
        ];
        let len = handles.len();
        for result in join_all(handles).await {
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
    Ok(())
}
