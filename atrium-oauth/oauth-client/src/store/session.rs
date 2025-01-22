use crate::types::TokenSet;
use atrium_api::types::string::Did;
use atrium_common::store::{memory::MemoryStore, Store};
use jose_jwk::Key;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub dpop_key: Key,
    pub token_set: TokenSet,
}

pub trait SessionStore: Store<Did, Arc<RwLock<Session>>> {}

pub type MemorySessionStore = MemoryStore<Did, Arc<RwLock<Session>>>;

impl SessionStore for MemorySessionStore {}
