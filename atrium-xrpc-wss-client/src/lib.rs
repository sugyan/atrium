mod client;
pub use client::{DefaultClient, Error};

pub mod subscriptions;

pub use atrium_xrpc_wss; // Re-export the atrium_xrpc_wss crate