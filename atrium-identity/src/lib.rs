#![doc = include_str!("../README.md")]
pub mod did;
mod error;
pub mod handle;
pub mod identity_resolver;

pub use self::error::{Error, Result};
