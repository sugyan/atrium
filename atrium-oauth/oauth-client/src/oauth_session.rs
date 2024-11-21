use std::fmt::Debug;

use atrium_api::{agent::SessionManager, types::string::Did};
use atrium_common::store::MapStore;
use atrium_identity::{did::DidResolver, handle::HandleResolver};
use atrium_xrpc::{
    http::{Request, Response},
    types::AuthorizationToken,
    HttpClient, XrpcClient,
};
use chrono::TimeDelta;
use thiserror::Error;

use crate::{server_agent::OAuthServerAgent, store::session::Session};

#[derive(Clone, Debug, Error)]
pub enum Error {}

pub struct OAuthSession<S, T, D, H>
where
    S: MapStore<(), Session> + Default,
    T: HttpClient + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
{
    session_store: S,
    server: OAuthServerAgent<T, D, H>,
}

impl<S, T, D, H> OAuthSession<S, T, D, H>
where
    S: MapStore<(), Session> + Default,
    S::Error: Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
{
    pub fn new(session_store: S, server: OAuthServerAgent<T, D, H>) -> Self {
        Self { session_store, server }
    }
    pub async fn get_session(&self, refresh: bool) -> crate::Result<Session> {
        let Some(session) = self
            .session_store
            .get(&())
            .await
            .map_err(|e| crate::Error::SessionStore(Box::new(e)))?
        else {
            panic!("a session should always exist");
        };
        if session.expires_in().expect("no expires_at") == TimeDelta::zero() && refresh {
            let token_set = self.server.refresh(session.token_set.clone()).await?;
            Ok(Session { dpop_key: session.dpop_key.clone(), token_set })
        } else {
            Ok(session)
        }
    }
    pub async fn logout(&self) -> crate::Result<()> {
        let session = self.get_session(false).await?;

        self.server.revoke(&session.token_set.access_token).await;
        self.session_store.clear().await.map_err(|e| crate::Error::SessionStore(Box::new(e)))?;

        Ok(())
    }
}

impl<S, T, D, H> HttpClient for OAuthSession<S, T, D, H>
where
    S: MapStore<(), Session> + Default + Sync,
    T: HttpClient + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.server.send_http(request).await
    }
}

impl<S, T, D, H> XrpcClient for OAuthSession<S, T, D, H>
where
    S: MapStore<(), Session> + Default + Sync,
    S::Error: Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
{
    fn base_uri(&self) -> String {
        // let Ok(Some(Session { dpop_key: _, token_set })) =
        //     futures::FutureExt::now_or_never(self.get_session(false)).transpose()
        // else {
        //     panic!("session, now or never");
        // };

        todo!()
    }
    async fn authorization_token(&self, is_refresh: bool) -> Option<AuthorizationToken> {
        let Ok(Session { dpop_key: _, token_set }) = self.get_session(false).await else {
            return None;
        };
        if is_refresh {
            token_set.refresh_token.as_ref().cloned().map(AuthorizationToken::Dpop)
        } else {
            Some(AuthorizationToken::Bearer(token_set.access_token.clone()))
        }
    }
}

impl<S, T, D, H> SessionManager for OAuthSession<S, T, D, H>
where
    S: MapStore<(), Session> + Default + Sync,
    S::Error: Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
{
    async fn did(&self) -> Option<Did> {
        let Ok(Some(session)) = self.session_store.get(&()).await else {
            return None;
        };
        Some(session.token_set.sub.parse().expect("TokenSet contains valid sub"))
    }
}
