use atrium_common::store::{memory::MemoryStore, Store};
use jose_jwk::Key;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InternalStateData {
    pub iss: String,
    pub dpop_key: Key,
    pub verifier: String,
    pub app_state: Option<String>,
}

pub trait StateStore: Store<String, InternalStateData> {}

pub type MemoryStateStore = MemoryStore<String, InternalStateData>;

impl StateStore for MemoryStateStore {}
