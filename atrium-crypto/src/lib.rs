#![doc = include_str!("../README.md")]
mod algorithm;
pub mod did;
mod encoding;
mod error;
pub mod keypair;
pub mod verify;

pub use crate::algorithm::Algorithm;
pub use crate::error::{Error, Result};
pub use multibase;

const DID_KEY_PREFIX: &str = "did:key:";
