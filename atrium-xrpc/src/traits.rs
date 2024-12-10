use crate::error::Error;
use crate::error::{XrpcError, XrpcErrorKind};
use crate::types::{AuthorizationToken, Header, NSID_REFRESH_SESSION};
use crate::{InputDataOrBytes, OutputDataOrBytes, XrpcRequest};
use http::{Method, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, future::Future};

/// An abstract HTTP client.
#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait HttpClient {
    /// Send an HTTP request and return the response.
    fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> impl Future<
        Output = core::result::Result<
            Response<Vec<u8>>,
            Box<dyn std::error::Error + Send + Sync + 'static>,
        >,
    >;
}

type XrpcResult<O, E> = core::result::Result<OutputDataOrBytes<O>, self::Error<E>>;

/// An abstract XRPC client.
///
/// [`send_xrpc()`](XrpcClient::send_xrpc) method has a default implementation,
/// which wraps the [`HttpClient::send_http()`]` method to handle input and output as an XRPC Request.
#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait XrpcClient: HttpClient {
    /// The base URI of the XRPC server.
    fn base_uri(&self) -> String;
    /// Get the authorization token to use `Authorization` header.
    #[allow(unused_variables)]
    fn authorization_token(
        &self,
        is_refresh: bool,
    ) -> impl Future<Output = Option<AuthorizationToken>> {
        async { None }
    }
    /// Get the `atproto-proxy` header.
    fn atproto_proxy_header(&self) -> impl Future<Output = Option<String>> {
        async { None }
    }
    /// Get the `atproto-accept-labelers` header.
    fn atproto_accept_labelers_header(&self) -> impl Future<Output = Option<Vec<String>>> {
        async { None }
    }
    /// Send an XRPC request and return the response.
    #[cfg(not(target_arch = "wasm32"))]
    fn send_xrpc<P, I, O, E>(
        &self,
        request: &XrpcRequest<P, I>,
    ) -> impl Future<Output = XrpcResult<O, E>>
    where
        P: Serialize + Send + Sync,
        I: Serialize + Send + Sync,
        O: DeserializeOwned + Send + Sync,
        E: DeserializeOwned + Send + Sync + Debug,
        // This code is duplicated because of this trait bound.
        // `Self` has to be `Sync` for `Future` to be `Send`.
        Self: Sync,
    {
        send_xrpc(self, request)
    }
    #[cfg(target_arch = "wasm32")]
    fn send_xrpc<P, I, O, E>(
        &self,
        request: &XrpcRequest<P, I>,
    ) -> impl Future<Output = XrpcResult<O, E>>
    where
        P: Serialize + Send + Sync,
        I: Serialize + Send + Sync,
        O: DeserializeOwned + Send + Sync,
        E: DeserializeOwned + Send + Sync + Debug,
    {
        send_xrpc(self, request)
    }
}

#[inline(always)]
async fn send_xrpc<P, I, O, E, C: XrpcClient + ?Sized>(
    client: &C,
    request: &XrpcRequest<P, I>,
) -> XrpcResult<O, E>
where
    P: Serialize + Send + Sync,
    I: Serialize + Send + Sync,
    O: DeserializeOwned + Send + Sync,
    E: DeserializeOwned + Send + Sync + Debug,
{
    let mut uri = format!("{}/xrpc/{}", client.base_uri(), request.nsid);
    // Query parameters
    if let Some(p) = &request.parameters {
        serde_html_form::to_string(p).map(|qs| {
            uri += "?";
            uri += &qs;
        })?;
    };
    let mut builder = Request::builder().method(&request.method).uri(&uri);
    // Headers
    if let Some(encoding) = &request.encoding {
        builder = builder.header(Header::ContentType, encoding);
    }
    if let Some(token) = client
        .authorization_token(request.method == Method::POST && request.nsid == NSID_REFRESH_SESSION)
        .await
    {
        builder = builder.header(Header::Authorization, token);
    }
    if let Some(proxy) = client.atproto_proxy_header().await {
        builder = builder.header(Header::AtprotoProxy, proxy);
    }
    if let Some(accept_labelers) = client.atproto_accept_labelers_header().await {
        builder = builder.header(Header::AtprotoAcceptLabelers, accept_labelers.join(", "));
    }
    // Body
    let body = if let Some(input) = &request.input {
        match input {
            InputDataOrBytes::Data(data) => serde_json::to_vec(&data)?,
            InputDataOrBytes::Bytes(bytes) => bytes.clone(),
        }
    } else {
        Vec::new()
    };
    // Send
    let (parts, body) =
        client.send_http(builder.body(body)?).await.map_err(Error::HttpClient)?.into_parts();
    if parts.status.is_success() {
        if parts
            .headers
            .get(http::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map_or(false, |content_type| content_type.starts_with("application/json"))
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
