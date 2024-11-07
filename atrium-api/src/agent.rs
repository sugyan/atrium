pub mod atp_agent;
#[cfg(feature = "bluesky")]
pub mod bluesky;
mod inner;
mod session_manager;

use crate::{client::Service, types::string::Did};
// pub use atp_agent::{AtpAgent, CredentialSession};
pub use session_manager::SessionManager;
use std::sync::Arc;

/// Supported proxy targets.
#[cfg(feature = "bluesky")]
pub type AtprotoServiceType = self::bluesky::AtprotoServiceType;

#[cfg(not(feature = "bluesky"))]
pub enum AtprotoServiceType {
    AtprotoLabeler,
}

#[cfg(not(feature = "bluesky"))]
impl AsRef<str> for AtprotoServiceType {
    fn as_ref(&self) -> &str {
        match self {
            Self::AtprotoLabeler => "atproto_labeler",
        }
    }
}

pub struct Agent<M>
where
    M: SessionManager + Send + Sync,
{
    session_manager: Arc<inner::Wrapper<M>>,
    pub api: Service<inner::Wrapper<M>>,
}

impl<M> Agent<M>
where
    M: SessionManager + Send + Sync,
{
    pub fn new(session_manager: M) -> Self {
        let session_manager = Arc::new(inner::Wrapper::new(session_manager));
        let api = Service::new(session_manager.clone());
        Self { session_manager, api }
    }
    pub async fn did(&self) -> Option<Did> {
        self.session_manager.did().await
    }
}
