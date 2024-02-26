#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg_attr(docsrs, doc(cfg(feature = "isahc")))]
#[cfg(feature = "isahc")]
pub mod isahc;
#[cfg_attr(docsrs, doc(cfg(feature = "reqwest")))]
#[cfg(any(feature = "reqwest", target_arch = "wasm32"))]
pub mod reqwest;
#[cfg_attr(docsrs, doc(cfg(feature = "surf")))]
#[cfg(feature = "surf")]
pub mod surf;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
