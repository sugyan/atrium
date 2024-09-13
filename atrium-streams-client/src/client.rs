//! This file provides a client for the `ATProto` XRPC over WSS protocol.
//! It implements the [`EventStreamClient`] trait for the [`WssClient`] struct.

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

use atrium_streams::client::{EventStreamClient, XrpcUri};

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
pub struct WssClient<'a, P: Serialize> {
    xrpc_uri: XrpcUri<'a>,
    params: Option<P>,
}

type StreamKind = WebSocketStream<MaybeTlsStream<TcpStream>>;
impl<P: Serialize + Send + Sync> EventStreamClient<<StreamKind as Stream>::Item, Error>
    for WssClient<'_, P>
{
    async fn connect(&self) -> Result<impl Stream<Item = <StreamKind as Stream>::Item>, Error> {
        let Self { xrpc_uri, params } = self;
        let mut uri = xrpc_uri.to_uri();
        //// Query parameters
        if let Some(p) = &params {
            uri.push('?');
            uri += &serde_html_form::to_string(p)?;
        };
        ////

        //// Request
        // Extracting the authority from the URI to set the Host header.
        let uri = Uri::from_str(&uri).map_err(|_| Error::InvalidUri)?;
        let authority = uri.authority().ok_or_else(|| Error::InvalidUri)?.as_str();
        let host = authority
            .find('@')
            .map_or_else(|| authority, |idx| authority.split_at(idx + 1).1);

        // Building the request.
        let mut request = Request::builder()
            .uri(&uri)
            .method("GET")
            .header("Host", host)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_key());

        // Adding the ATProto headers.
        if let Some(proxy) = self.atproto_proxy_header().await {
            request = request.header(Header::AtprotoProxy, proxy);
        }
        if let Some(accept_labelers) = self.atproto_accept_labelers_header().await {
            request = request.header(Header::AtprotoAcceptLabelers, accept_labelers.join(", "));
        }

        // In our case, the only thing that could possibly fail is the URI. The headers are all `String`/`&str`.
        let request = request.body(()).map_err(|_| Error::InvalidUri)?;
        ////

        let (stream, _) = connect_async(request).await?;
        Ok(stream)
    }
}
