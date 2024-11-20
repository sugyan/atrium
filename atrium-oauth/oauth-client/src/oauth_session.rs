use crate::{DpopClient, TokenSet};
use atrium_api::{agent::SessionManager, types::string::Did};
use atrium_common::store::{memory::MemoryMapStore, MapStore};
use atrium_xrpc::{
    http::{Request, Response},
    types::AuthorizationToken,
    HttpClient, XrpcClient,
};

pub struct OAuthSession<T, S = MemoryMapStore<String, String>>
where
    S: MapStore<String, String>,
{
    inner: DpopClient<T, S>,
    token_set: TokenSet, // TODO: replace with a session store?
}

impl<T, S> OAuthSession<T, S>
where
    S: MapStore<String, String> + Send + Sync + 'static,
{
    pub fn new(session_store: S, server: OAuthServerAgent<T, D, H>) -> Self {
        Self { session_store, server }
    }
    pub async fn get_session(&self, refresh: bool) -> crate::Result<Session> {
        let Some(session) = self.session_store.get().await.expect("todo") else {
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
        self.session_store.clear().await.expect("todo");

        Ok(())
    }
}

impl<T, S> HttpClient for OAuthSession<T, S>
where
    T: HttpClient + Send + Sync + 'static,
    S: MapStore<String, String> + Send + Sync + 'static,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.inner.send_http(request).await
    }
}

impl<T, S> XrpcClient for OAuthSession<T, S>
where
    T: HttpClient + Send + Sync + 'static,
    S: MapStore<String, String> + Send + Sync + 'static,
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

impl<T, S> SessionManager for OAuthSession<T, S>
where
    T: HttpClient + Send + Sync + 'static,
    S: MapStore<String, String> + Send + Sync + 'static,
{
    async fn did(&self) -> Option<Did> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
