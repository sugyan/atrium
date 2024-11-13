use crate::store::{memory::MemorySimpleStore, SimpleStore};
use crate::{DpopClient, TokenSet};
use atrium_api::{agent::SessionManager, types::string::Did};
use atrium_xrpc::types::AuthorizationType;
use atrium_xrpc::{
    http::{Request, Response},
    HttpClient, XrpcClient,
};

pub struct OAuthSession<T, S = MemorySimpleStore<String, String>>
where
    S: SimpleStore<String, String>,
{
    inner: DpopClient<T, S>,
    token_set: TokenSet, // TODO: replace with a session store?
}

impl<T, S> OAuthSession<T, S>
where
    S: SimpleStore<String, String> + Send + Sync + 'static,
{
    pub fn new(dpop_client: DpopClient<T, S>, token_set: TokenSet) -> Self {
        Self { inner: dpop_client, token_set }
    }
}

impl<T, S> HttpClient for OAuthSession<T, S>
where
    T: HttpClient + Send + Sync + 'static,
    S: SimpleStore<String, String> + Send + Sync + 'static,
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
    S: SimpleStore<String, String> + Send + Sync + 'static,
{
    fn base_uri(&self) -> String {
        self.token_set.aud.clone()
    }
    fn authorization_type(&self) -> AuthorizationType {
        AuthorizationType::Dpop
    }
    async fn authorization_token(&self, is_refresh: bool) -> Option<String> {
        Some(self.token_set.access_token.clone())
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
    S: SimpleStore<String, String> + Send + Sync + 'static,
{
    async fn did(&self) -> Option<Did> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
