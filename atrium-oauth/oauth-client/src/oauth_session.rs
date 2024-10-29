use std::sync::Arc;

use atrium_api::types::string::Did;
use atrium_identity::{did::DidResolver, handle::HandleResolver};
use atrium_xrpc::HttpClient;
use thiserror::Error;

use crate::{
    server_agent::OAuthServerAgent,
    store::session::{Session, SessionStore},
    types::TokenInfo,
    Result, TokenSet,
};

#[derive(Clone, Debug, Error)]
pub enum Error {}

pub struct OAuthSession<S, T, D, H>
where
    S: SessionStore,
    T: HttpClient + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
{
    session_store: S,
    pub server: Arc<OAuthServerAgent<T, D, H>>,
    pub sub: Did,
}
impl<S, T, D, H> OAuthSession<S, T, D, H>
where
    S: SessionStore,
    T: HttpClient + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
{
    pub fn new(server: OAuthServerAgent<T, D, H>, sub: Did, session_store: S) -> Self {
        Self { server: Arc::new(server), sub, session_store }
    }

    pub async fn get_token_set(&self, _refresh: Option<bool>) -> Result<TokenSet> {
        let Some(value) = self.session_store.get(&self.sub).await.unwrap() else { todo!() };

        let server = self.server.clone();

        let get_cached = value.get_cached(|session| {
            Box::pin(async move {
                let Some(session) = session else { todo!() };

                Ok(Session {
                    dpop_key: session.dpop_key,
                    token_set: server.refresh(session.token_set.clone()).await.unwrap(),
                })
            })
        });
        let session = get_cached.await.unwrap();

        Ok(session.token_set)
    }

    pub async fn get_token_info(&self, refresh: Option<bool>) -> Result<TokenInfo> {
        let TokenSet { iss, sub, aud, scope, expires_at, .. } = self.get_token_set(refresh).await?;
        let expires_at = expires_at.as_ref().map(AsRef::as_ref).cloned();

        Ok(TokenInfo::new(iss, sub.parse().expect("valid Did"), aud, scope, expires_at))
    }

    pub async fn logout(&self, _refresh: Option<bool>) -> Result<()> {
        let token_set = self.get_token_set(Some(false)).await?;

        self.server.revoke(&token_set.access_token).await?;

        let _ = self.session_store.del(&self.sub).await;
        Ok(())
    }
}
