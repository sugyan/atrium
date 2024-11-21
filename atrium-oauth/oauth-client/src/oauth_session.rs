use crate::http_client::dpop::Error;
use crate::server_agent::OAuthServerAgent;
use crate::store::{memory::MemorySimpleStore, SimpleStore};
use crate::{DpopClient, TokenSet};
use atrium_api::{agent::SessionManager, types::string::Did};
use atrium_xrpc::{
    http::{Request, Response},
    types::AuthorizationToken,
    HttpClient, XrpcClient,
};
use jose_jwk::Key;
use std::sync::Arc;

pub struct OAuthSession<T, D, H, S = MemorySimpleStore<String, String>>
where
    T: HttpClient + Send + Sync + 'static,
    S: SimpleStore<String, String>,
{
    server_agent: OAuthServerAgent<T, D, H>,
    dpop_client: DpopClient<T, S>,
    token_set: TokenSet, // TODO: replace with a session store?
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
}

impl<T, D, H, S> HttpClient for OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync + 'static,
    D: Send + Sync + 'static,
    H: Send + Sync + 'static,
    S: SimpleStore<String, String> + Send + Sync + 'static,
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
    S: SimpleStore<String, String> + Send + Sync + 'static,
{
    fn base_uri(&self) -> String {
        self.token_set.aud.clone()
    }
    async fn authorization_token(&self, _is_refresh: bool) -> Option<AuthorizationToken> {
        Some(AuthorizationToken::Dpop(self.token_set.access_token.clone()))
    }
    // async fn atproto_proxy_header(&self) -> Option<String> {
    //     todo!()
    // }
    // async fn atproto_accept_labelers_header(&self) -> Option<Vec<String>> {
    //     todo!()
    // }
    // async fn send_xrpc<P, I, O, E>(
    //     &self,
    //     request: &XrpcRequest<P, I>,
    // ) -> Result<OutputDataOrBytes<O>, Error<E>>
    // where
    //     P: Serialize + Send + Sync,
    //     I: Serialize + Send + Sync,
    //     O: DeserializeOwned + Send + Sync,
    //     E: DeserializeOwned + Send + Sync + Debug,
    // {
    //     todo!()
    // }
}

impl<T, D, H, S> SessionManager for OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync + 'static,
    D: Send + Sync + 'static,
    H: Send + Sync + 'static,
    S: SimpleStore<String, String> + Send + Sync + 'static,
{
    async fn did(&self) -> Option<Did> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
