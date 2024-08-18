mod appview_resolver;

pub use appview_resolver::AppViewResolver;
use async_trait::async_trait;
use atrium_api::types::string::{Did, Handle};
use atrium_xrpc::http::uri::InvalidUri;
use atrium_xrpc::http::Uri;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] atrium_xrpc::http::Error),
    #[error("http client error: {0}")]
    HttpClient(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeHtmlForm(#[from] serde_html_form::ser::Error),
    #[error("status: {0:?}")]
    Status(Option<&'static str>),
}

pub type Result<T> = core::result::Result<T, Error>;

#[async_trait]
pub trait HandleResolver: Send + Sync + 'static {
    async fn resolve(&self, handle: &Handle) -> Result<Did>;
}

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
