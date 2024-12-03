use std::sync::Arc;

use atrium_api::{agent::SessionManager, types::string::Did};
use atrium_common::store::{memory::MemoryStore, Store};
use atrium_identity::{did::DidResolver, handle::HandleResolver};
use atrium_xrpc::{
    http::{Request, Response},
    types::AuthorizationToken,
    HttpClient, XrpcClient,
};
use jose_jwk::Key;

use crate::{
    http_client::dpop::Error,
    server_agent::OAuthServerAgent,
    store::session::{MemorySessionStore, SessionStore},
    DpopClient, TokenSet,
};

pub struct OAuthSession<
    T,
    D,
    H,
    S0 = MemoryStore<String, String>,
    S1 = MemorySessionStore<(), TokenSet>,
> where
    T: HttpClient + Send + Sync + 'static,
    S0: Store<String, String>,
    S1: SessionStore<(), TokenSet>,
{
    #[allow(dead_code)]
    server_agent: OAuthServerAgent<T, D, H>,
    dpop_client: DpopClient<T, S0>,
    session_store: S1,
}

impl<T, D, H> OAuthSession<T, D, H>
where
    T: HttpClient + Send + Sync + 'static,
{
    pub(crate) async fn new(
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

        let session_store = MemorySessionStore::default();
        session_store.set((), token_set).await.map_err(|e| Error::SessionStore(Box::new(e)))?;

        Ok(Self { server_agent, dpop_client, session_store })
    }
    pub fn dpop_key(&self) -> Key {
        self.dpop_client.key.clone()
    }
    pub async fn token_set(&self) -> Result<TokenSet, Error> {
        let token_set =
            self.session_store.get(&()).await.map_err(|e| Error::SessionStore(Box::new(e)))?;
        Ok(token_set.expect("session store can never be empty"))
    }
}

impl<T, D, H> OAuthSession<T, D, H>
where
    T: HttpClient + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
{
    pub async fn refresh(&self) -> Result<(), Error> {
        let Some(token_set) =
            self.session_store.get(&()).await.map_err(|e| Error::SessionStore(Box::new(e)))?
        else {
            return Ok(());
        };
        let Ok(token_set) = self.server_agent.refresh(&token_set).await else {
            todo!();
        };

        self.session_store.set((), token_set).await.map_err(|e| Error::SessionStore(Box::new(e)))
    }
    pub async fn logout(&self) -> Result<(), Error> {
        let Some(token_set) =
            self.session_store.get(&()).await.map_err(|e| Error::SessionStore(Box::new(e)))?
        else {
            return Ok(());
        };
        self.server_agent.revoke(&token_set.access_token).await;

        self.session_store.clear().await.map_err(|e| Error::SessionStore(Box::new(e)))
    }
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
        // self.token_set.aud.clone()
        todo!()
    }
    async fn authorization_token(&self, is_refresh: bool) -> Option<AuthorizationToken> {
        let token_set = self.session_store.get(&()).await.transpose().and_then(Result::ok)?;
        if is_refresh {
            token_set.refresh_token.as_ref().cloned().map(AuthorizationToken::Dpop)
        } else {
            Some(AuthorizationToken::Dpop(token_set.access_token.clone()))
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
        let token_set = self.session_store.get(&()).await.transpose().and_then(Result::ok)?;
        Some(token_set.sub.clone())
    }
}
