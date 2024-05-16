#![doc = include_str!("../README.md")]
mod algorithm;
pub mod did;
mod encoding;
pub mod error;
pub mod keypair;
pub mod verify;

pub use algorithm::Algorithm;
pub use multibase;

const DID_KEY_PREFIX: &str = "did:key:";
