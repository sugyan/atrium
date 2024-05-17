use http::Method;
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

/// A type which can be used as a return value of [`XrpcClient::send_xrpc()`](crate::XrpcClient::send_xrpc).
///
/// JSON deserializable data or raw bytes.
pub enum OutputDataOrBytes<T>
where
    T: DeserializeOwned,
{
    Data(T),
    Bytes(Vec<u8>),
}

/// A request which can be executed with [`XrpcClient::send_xrpc()`](crate::XrpcClient::send_xrpc).
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
