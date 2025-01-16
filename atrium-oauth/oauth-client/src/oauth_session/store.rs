use atrium_api::agent::AuthorizationProvider;
use atrium_common::store::{self, memory::MemoryStore, Store};
use atrium_xrpc::types::AuthorizationToken;

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait OAuthSessionStore: store::Store<(), String> + AuthorizationProvider {}

#[derive(Default)]
pub struct MemorySessionStore(MemoryStore<(), String>);

impl OAuthSessionStore for MemorySessionStore {}

impl Store<(), String> for MemorySessionStore {
    type Error = store::memory::Error;

    async fn get(&self, key: &()) -> Result<Option<String>, Self::Error> {
        self.0.get(key).await
    }
    async fn set(&self, key: (), value: String) -> Result<(), Self::Error> {
        self.0.set(key, value).await
    }
    async fn del(&self, key: &()) -> Result<(), Self::Error> {
        self.0.del(key).await
    }
    async fn clear(&self) -> Result<(), Self::Error> {
        self.0.clear().await
    }
}

impl AuthorizationProvider for MemorySessionStore {
    async fn authorization_token(&self, _: bool) -> Option<AuthorizationToken> {
        self.0.get(&()).await.ok().flatten().map(AuthorizationToken::Dpop)
    }
}
