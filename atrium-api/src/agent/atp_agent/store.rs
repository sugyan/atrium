use super::AtpSession;
use crate::agent::AuthorizationProvider;
use atrium_common::store::{memory::MemoryStore, Store};
use atrium_xrpc::types::AuthorizationToken;

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait AtpSessionStore: Store<(), AtpSession> + AuthorizationProvider {}

pub type MemorySessionStore = MemoryStore<(), AtpSession>;

impl AtpSessionStore for MemorySessionStore {}

impl AuthorizationProvider for MemorySessionStore {
    async fn authorization_token(&self, is_refresh: bool) -> Option<AuthorizationToken> {
        self.get(&())
            .await
            .ok()
            .flatten()
            .map(
                |session| {
                    if is_refresh {
                        session.refresh_jwt.clone()
                    } else {
                        session.access_jwt.clone()
                    }
                },
            )
            .map(AuthorizationToken::Bearer)
    }
}
