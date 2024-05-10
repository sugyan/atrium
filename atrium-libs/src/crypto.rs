mod algorithm;
pub mod did;
pub mod error;
mod utils;

pub use algorithm::Algorithm;

const DID_KEY_PREFIX: &str = "did:key:";
