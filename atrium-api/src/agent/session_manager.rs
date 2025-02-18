use crate::types::string::Did;
use atrium_xrpc::XrpcClient;
use std::future::Future;

/// A trait for managing sessions.
///
/// [`Agent`](crate::agent::Agent) creation requires an implementation of this `SessionManager`.
#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait SessionManager: XrpcClient {
    fn did(&self) -> impl Future<Output = Option<Did>>;
}
