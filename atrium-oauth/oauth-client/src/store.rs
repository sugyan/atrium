use atrium_common::store::{memory::MemoryMapStore, MapStore};
use jose_jwk::Key;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InternalStateData {
    pub iss: String,
    pub dpop_key: Key,
    pub verifier: String,
}

pub trait StateStore: MapStore<String, InternalStateData> {}

pub type MemoryStateStore = MemoryMapStore<String, InternalStateData>;

impl StateStore for MemoryStateStore {}
