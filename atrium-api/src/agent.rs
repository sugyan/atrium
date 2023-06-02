//! An ATP "Agent".
//! Manages session token lifecycles and provides all XRPC methods.
use crate::xrpc::{HttpClient, XrpcClient};
use async_trait::async_trait;
use http::{Request, Response};
use std::error::Error;

/// Type alias for the [com::atproto::server::create_session::Output](crate::com::atproto::server::create_session::Output)
pub type Session = crate::com::atproto::server::create_session::Output;

pub struct AtpAgent<T>
where
    T: XrpcClient,
{
    api: T,
    session: Option<Session>,
}

impl<T> AtpAgent<T>
where
    T: XrpcClient,
{
    pub fn new(api: T) -> Self {
        Self { api, session: None }
    }
    pub fn set_session(&mut self, session: Session) {
        self.session = Some(session);
    }
}

#[async_trait]
impl<T> HttpClient for AtpAgent<T>
where
    T: XrpcClient + Send + Sync,
{
    async fn send(
        &self,
        req: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn Error + Send + Sync + 'static>> {
        HttpClient::send(&self.api, req).await
    }
}

#[async_trait]
impl<T> XrpcClient for AtpAgent<T>
where
    T: XrpcClient + Send + Sync,
{
    fn host(&self) -> &str {
        self.api.host()
    }
    fn auth(&self, is_refresh: bool) -> Option<&str> {
        self.session.as_ref().map(|s| {
            if is_refresh {
                s.refresh_jwt.as_str()
            } else {
                s.access_jwt.as_str()
            }
        })
    }
}
