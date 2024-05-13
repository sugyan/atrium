#![doc = include_str!("../README.md")]
mod algorithm;
pub mod did;
pub mod error;
pub mod keypair;

pub use algorithm::Algorithm;

const DID_KEY_PREFIX: &str = "did:key:";
