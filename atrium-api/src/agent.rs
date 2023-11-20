//! An ATP "Agent".
//! Manages session token lifecycles and provides all XRPC methods.
mod inner;
pub mod store;

use self::store::SessionStore;
use crate::client::Service;
use atrium_xrpc::error::Error;
use atrium_xrpc::XrpcClient;
use std::sync::Arc;

/// Type alias for the [com::atproto::server::create_session::Output](crate::com::atproto::server::create_session::Output)
pub type Session = crate::com::atproto::server::create_session::Output;

pub struct AtpAgent<S, T>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    store: Arc<S>,
    pub api: Service<inner::Inner<S, T>>,
}

impl<S, T> AtpAgent<S, T>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    /// Create a new agent.
    pub fn new(xrpc: T, store: S) -> Self {
        let store = Arc::new(store);
        let api = Service::new(Arc::new(inner::Inner::new(Arc::clone(&store), xrpc)));
        Self { api, store }
    }
    /// Start a new session with this agent.
    pub async fn login(
        &self,
        identifier: &str,
        password: &str,
    ) -> Result<Session, Error<crate::com::atproto::server::create_session::Error>> {
        let result = self
            .api
            .com
            .atproto
            .server
            .create_session(crate::com::atproto::server::create_session::Input {
                identifier: identifier.into(),
                password: password.into(),
            })
            .await?;
        self.store.set_session(result.clone()).await;
        Ok(result)
    }
    /// Resume a pre-existing session with this agent.
    pub async fn resume_session(
        &self,
        session: Session,
    ) -> Result<(), Error<crate::com::atproto::server::get_session::Error>> {
        self.store.set_session(session.clone()).await;
        let result = self.api.com.atproto.server.get_session().await;
        match result {
            Ok(output) => {
                assert_eq!(output.did, session.did);
                if let Some(mut session) = self.store.get_session().await {
                    session.did_doc = output.did_doc;
                    session.email = output.email;
                    session.email_confirmed = output.email_confirmed;
                    session.handle = output.handle;
                    self.store.set_session(session).await;
                }
                Ok(())
            }
            Err(err) => {
                self.store.clear_session().await;
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests;
