use std::net::{Ipv4Addr, SocketAddr};

use atrium_streams::{atrium_api::com::atproto::sync::subscribe_repos, client::EventStreamClient};
use atrium_xrpc::http::{header::SEC_WEBSOCKET_KEY, HeaderMap, HeaderValue};
use futures::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    runtime::Runtime,
};
use tokio_tungstenite::{
    tungstenite::{
        handshake::server::{ErrorResponse, Request, Response},
        Message,
    },
    WebSocketStream,
};

use crate::WssClient;

use super::{gen_request, get_host};

#[test]
fn client() {
    let fut = async {
        let ipv4 = Ipv4Addr::LOCALHOST.to_string();
        let xrpc_uri = format!("ws://{ipv4}:3000/xrpc/{}", subscribe_repos::NSID);
        let (client, mut client_headers) = wss_client(&xrpc_uri).await;

        let server_handle = tokio::spawn(mock_wss_server());
        let mut client_stream = client.connect(xrpc_uri).await.unwrap();
        let (server_stream, mut server_headers, route) = server_handle.await.unwrap();

        assert_eq!(route, format!("/xrpc/{}", subscribe_repos::NSID));

        client_headers.remove(SEC_WEBSOCKET_KEY);
        server_headers.remove(SEC_WEBSOCKET_KEY);
        assert_eq!(client_headers, server_headers);

        let (mut inbound, _) = server_stream.split();
        inbound.send(Message::text("test_message")).await.unwrap();
        let msg = client_stream.next().await.unwrap().unwrap();
        assert_eq!(msg, Message::text("test_message"));
    };
    Runtime::new().unwrap().block_on(fut);
}

async fn wss_client(
    uri: &str,
) -> (WssClient<subscribe_repos::ParametersData>, HeaderMap<HeaderValue>) {
    let params = subscribe_repos::ParametersData { cursor: None };

    let client = WssClient::builder().params(params).build();

    let (uri, host) = get_host(uri).unwrap();
    let req = gen_request(&client, &uri, &host).await.unwrap();
    let headers = req.headers();

    (client, headers.clone())
}

async fn mock_wss_server() -> (WebSocketStream<TcpStream>, HeaderMap, String) {
    let sock_addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 3000));

    let listener = TcpListener::bind(sock_addr).await.expect("Failed to bind to port!");

    let headers: HeaderMap;
    let route: String;
    let (stream, _) = listener.accept().await.unwrap();
    let (headers_, route_, stream) = extract_headers(stream).await;
    headers = headers_;
    route = route_;

    (stream, headers, route)
}

async fn extract_headers(
    raw_stream: TcpStream,
) -> (HeaderMap<HeaderValue>, String, WebSocketStream<TcpStream>) {
    let mut headers: Option<HeaderMap<HeaderValue>> = None;
    let mut route: Option<String> = None;

    let copy_headers_callback =
        |request: &Request, response: Response| -> Result<Response, ErrorResponse> {
            headers = Some(request.headers().clone());
            route = Some(request.uri().path().to_owned());
            Ok(response)
        };

    let stream = tokio_tungstenite::accept_hdr_async(raw_stream, copy_headers_callback)
        .await
        .expect("Error during the websocket handshake occurred");

    (headers.unwrap(), route.unwrap(), stream)
}
