use super::memory::MemorySimpleStore;
use super::SimpleStore;
use elliptic_curve::JwkEcKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalStateData {
    pub iss: String,
    pub dpop_key: JwkEcKey,
    pub verifier: String,
}

pub trait StateStore: SimpleStore<String, InternalStateData> {}

impl StateStore for MemorySimpleStore<String, InternalStateData> {}
