use crate::types::TokenSet;
use atrium_api::types::string::Did;
use atrium_common::store::{memory::MemoryStore, Store};
use jose_jwk::Key;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub dpop_key: Key,
    pub token_set: TokenSet,
}

pub trait SessionStore: Store<Did, Session> {}

pub type MemorySessionStore = MemoryStore<Did, Session>;

impl SessionStore for MemorySessionStore {}
