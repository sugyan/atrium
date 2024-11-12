use atrium_common::store::{memory::MemoryMapStore, MapStore};
use jose_jwk::Key;
use serde::{Deserialize, Serialize};

use crate::TokenSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub dpop_key: Key,
    pub token_set: TokenSet,
}

pub trait SessionStore: MapStore<String, Session> {}

pub type MemorySessionStore = MemoryMapStore<String, Session>;

impl SessionStore for MemorySessionStore {}
