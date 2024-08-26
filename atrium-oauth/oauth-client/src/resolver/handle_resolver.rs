mod appview_resolver;

use super::Resolver;
pub use appview_resolver::AppViewResolver;
use atrium_api::types::string::{Did, Handle};
use atrium_xrpc::http::uri::InvalidUri;
use atrium_xrpc::http::Uri;
use std::fmt::Debug;
use std::sync::Arc;

pub trait HandleResolver: Resolver<Input = Handle, Output = Did> {}

pub enum HandleResolverConfig {
    AppView(Uri),
    Service(Arc<dyn HandleResolver>),
}

impl Debug for HandleResolverConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AppView(arg) => f.debug_tuple("AppView").field(arg).finish(),
            Self::Service(_) => f.debug_tuple("Service").finish(),
        }
    }
}

impl TryFrom<&str> for HandleResolverConfig {
    type Error = InvalidUri;

    fn try_from(value: &str) -> core::result::Result<Self, Self::Error> {
        Ok(Self::AppView(Uri::try_from(value)?))
    }
}
