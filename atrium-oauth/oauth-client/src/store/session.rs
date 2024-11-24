use crate::types::TokenSet;
use atrium_api::types::string::{Datetime, Did};
use atrium_common::store::{memory::MemoryStore, Store};
use chrono::TimeDelta;
use jose_jwk::Key;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub dpop_key: Key,
    pub token_set: TokenSet,
}

impl Session {
    pub fn expires_in(&self) -> Option<TimeDelta> {
        self.token_set.expires_at.as_ref().map(Datetime::as_ref).map(|expires_at| {
            expires_at.signed_duration_since(Datetime::now().as_ref()).max(TimeDelta::zero())
        })
    }
}

pub trait SessionStore: Store<Did, Session> {}

pub type MemorySessionStore = MemoryStore<Did, Session>;

impl SessionStore for MemorySessionStore {}
