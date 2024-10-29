use atrium_api::types::string::Did;
use chrono::{DateTime, FixedOffset};
use jose_jwk::Key;
use serde::{Deserialize, Serialize};

use crate::{oauth_session, TokenSet};

use super::{
    cached::{Cached, Expired},
    memory::MemorySimpleStore,
    SimpleStore,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub dpop_key: Key,
    pub token_set: TokenSet,
}

impl Session {
    pub fn new(dpop_key: Key, token_set: TokenSet) -> Self {
        Self { dpop_key, token_set }
    }
}

impl Expired for Session {
    fn expires_at(&self) -> Option<DateTime<FixedOffset>> {
        self.token_set.expires_at.as_ref().map(AsRef::as_ref).cloned()
    }
}

pub trait SessionStore: SimpleStore<Did, Cached<Session, oauth_session::Error>> + Clone {}

pub type MemorySessionStore = MemorySimpleStore<Did, Cached<Session, oauth_session::Error>>;

impl SessionStore for MemorySessionStore {}
