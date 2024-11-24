use std::sync::Arc;

use atrium_api::{agent::SessionManager, types::string::Did};
use atrium_common::store::{memory::MemoryStore, Store};
use atrium_xrpc::{
    http::{Request, Response},
    types::AuthorizationToken,
    HttpClient, XrpcClient,
};
use jose_jwk::Key;

use crate::{http_client::dpop::Error, server_agent::OAuthServerAgent, DpopClient, TokenSet};

pub struct OAuthSession<T, D, H, S = MemoryStore<String, String>>
where
    T: HttpClient + Send + Sync + 'static,
    S: Store<String, String>,
{
    #[allow(dead_code)]
    server_agent: OAuthServerAgent<T, D, H>,
    dpop_client: DpopClient<T, S>,
    token_set: TokenSet,
}

impl<T, D, H> OAuthSession<T, D, H>
where
    T: HttpClient + Send + Sync + 'static,
{
    pub(crate) fn new(
        server_agent: OAuthServerAgent<T, D, H>,
        dpop_key: Key,
        http_client: Arc<T>,
        token_set: TokenSet,
    ) -> Result<Self, Error> {
        let dpop_client = DpopClient::new(
            dpop_key,
            http_client.clone(),
            false,
            &server_agent.server_metadata.token_endpoint_auth_signing_alg_values_supported,
        )?;
        Ok(Self { server_agent, dpop_client, token_set })
    }
    pub fn dpop_key(&self) -> Key {
        self.dpop_client.key.clone()
    }
    pub fn token_set(&self) -> TokenSet {
        self.token_set.clone()
    }
    // pub async fn get_session(&self, refresh: bool) -> crate::Result<Session> {
    //     let Some(session) = self
    //         .session_store
    //         .get(&())
    //         .await
    //         .map_err(|e| crate::Error::SessionStore(Box::new(e)))?
    //     else {
    //         panic!("a session should always exist");
    //     };
    //     if session.expires_in().expect("no expires_at") == TimeDelta::zero() && refresh {
    //         let token_set = self.server.refresh(session.token_set.clone()).await?;
    //         Ok(Session { dpop_key: session.dpop_key.clone(), token_set })
    //     } else {
    //         Ok(session)
    //     }
    // }
    // pub async fn logout(&self) -> crate::Result<()> {
    //     let session = self.get_session(false).await?;

    //     self.server.revoke(&session.token_set.access_token).await;
    //     self.session_store.clear().await.map_err(|e| crate::Error::SessionStore(Box::new(e)))?;

    //     Ok(())
    // }
}

impl<T, D, H, S> HttpClient for OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync + 'static,
    D: Send + Sync + 'static,
    H: Send + Sync + 'static,
    S: Store<String, String> + Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.dpop_client.send_http(request).await
    }
}

impl<T, D, H, S> XrpcClient for OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync + 'static,
    D: Send + Sync + 'static,
    H: Send + Sync + 'static,
    S: Store<String, String> + Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    fn base_uri(&self) -> String {
        self.token_set.aud.clone()
    }
    async fn authorization_token(&self, is_refresh: bool) -> Option<AuthorizationToken> {
        if is_refresh {
            self.token_set.refresh_token.as_ref().cloned().map(AuthorizationToken::Dpop)
        } else {
            Some(AuthorizationToken::Dpop(self.token_set.access_token.clone()))
        }
    }
}

impl<T, D, H, S> SessionManager for OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync + 'static,
    D: Send + Sync + 'static,
    H: Send + Sync + 'static,
    S: Store<String, String> + Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    async fn did(&self) -> Option<Did> {
        Some(self.token_set.sub.clone())
    }
}
