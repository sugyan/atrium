//! An ATP "Agent".
//! Manages session token lifecycles and provides all XRPC methods.
use async_trait::async_trait;
use atrium_xrpc::{HttpClient, XrpcClient};
use http::{Request, Response};
use std::error::Error;
use std::sync::{Arc, RwLock};

use crate::client::AtpServiceClient;

/// Type alias for the [com::atproto::server::create_session::Output](crate::com::atproto::server::create_session::Output)
pub type Session = crate::com::atproto::server::create_session::Output;

pub struct BaseClient<T>
where
    T: XrpcClient,
{
    xrpc: T,
    session: Arc<RwLock<Option<Session>>>,
}

#[async_trait]
impl<T> HttpClient for BaseClient<T>
where
    T: XrpcClient + Send + Sync,
{
    async fn send(
        &self,
        req: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn Error + Send + Sync + 'static>> {
        HttpClient::send(&self.xrpc, req).await
    }
}

#[async_trait]
impl<T> XrpcClient for BaseClient<T>
where
    T: XrpcClient + Send + Sync,
{
    fn host(&self) -> &str {
        self.xrpc.host()
    }
    fn auth(&self, is_refresh: bool) -> Option<String> {
        self.session.read().ok().and_then(|lock| {
            lock.as_ref().map(|session| {
                if is_refresh {
                    session.refresh_jwt.clone()
                } else {
                    session.access_jwt.clone()
                }
            })
        })
    }
}

pub struct AtpAgent<T>
where
    T: XrpcClient + Send + Sync,
{
    pub api: crate::client::AtpServiceClient<T>,
    session: Arc<RwLock<Option<Session>>>,
}

impl<T> AtpAgent<BaseClient<T>>
where
    T: XrpcClient + Send + Sync,
{
    pub fn new(xrpc: T) -> Self {
        let session = Arc::new(RwLock::new(None));
        let base = BaseClient {
            xrpc,
            session: Arc::clone(&session),
        };
        let api = AtpServiceClient::new(Arc::new(base));
        Self { api, session }
    }
    pub fn set_session(&mut self, session: Session) {
        if let Ok(mut lock) = self.session.write() {
            *lock = Some(session);
        }
    }
}
