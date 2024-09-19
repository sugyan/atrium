//! This file provides a client for the `ATProto` XRPC over WSS protocol.
//! It implements the [`EventStreamClient`] trait for the [`WssClient`] struct.

#[cfg(test)]
mod tests;

use std::str::FromStr;

use futures::Stream;
use tokio::net::TcpStream;

use atrium_xrpc::{
    http::{Request, Uri},
    types::Header,
};
use bon::Builder;
use serde::Serialize;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, handshake::client::generate_key},
    MaybeTlsStream, WebSocketStream,
};

use atrium_streams::client::EventStreamClient;

/// An enum of possible error kinds for this crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid uri")]
    InvalidUri,
    #[error("Parsing parameters failed: {0}")]
    ParsingParameters(#[from] serde_html_form::ser::Error),
    #[error("Connection error: {0}")]
    Connection(#[from] tungstenite::Error),
}

#[derive(Builder)]
pub struct WssClient<P: Serialize> {
    params: Option<P>,
}

type StreamKind = WebSocketStream<MaybeTlsStream<TcpStream>>;
impl<P: Serialize + Send + Sync> EventStreamClient<<StreamKind as Stream>::Item, Error>
    for WssClient<P>
{
    async fn connect(
        &self,
        mut uri: String,
    ) -> Result<impl Stream<Item = <StreamKind as Stream>::Item>, Error> {
        let Self { params } = self;

        // Query parameters
        if let Some(p) = &params {
            uri.push('?');
            uri += &serde_html_form::to_string(p)?;
        };

        // Request
        let (uri, host) = get_host(&uri)?;
        let request = gen_request(self, &uri, &*host).await?;

        // Connection
        let (stream, _) = connect_async(request).await?;
        Ok(stream)
    }
}

/// Extract the URI and host from a string.
fn get_host(uri: &str) -> Result<(Uri, Box<str>), Error> {
    let uri = Uri::from_str(uri).map_err(|_| Error::InvalidUri)?;
    let authority = uri.authority().ok_or_else(|| Error::InvalidUri)?.as_str();
    let host = authority.find('@').map_or_else(|| authority, |idx| authority.split_at(idx + 1).1);
    let host = Box::from(host);
    Ok((uri, host))
}

/// Generate a request for the given URI and host.
/// It sets the necessary headers for a WebSocket connection,
/// plus the client's `AtprotoProxy` and `AtprotoAcceptLabelers` headers.
async fn gen_request<P: Serialize + Send + Sync>(
    client: &WssClient<P>,
    uri: &Uri,
    host: &str,
) -> Result<Request<()>, Error> {
    let mut request = Request::builder()
        .uri(uri)
        .method("GET")
        .header("Host", host)
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", generate_key());
    if let Some(proxy) = client.atproto_proxy_header().await {
        request = request.header(Header::AtprotoProxy, proxy);
    }
    if let Some(accept_labelers) = client.atproto_accept_labelers_header().await {
        request = request.header(Header::AtprotoAcceptLabelers, accept_labelers.join(", "));
    }
    let request = request.body(()).map_err(|_| Error::InvalidUri)?;
    Ok(request)
}
