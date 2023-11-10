#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg_attr(docsrs, doc(cfg(feature = "isahc")))]
#[cfg(feature = "isahc")]
pub mod isahc;
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "reqwest-native", feature = "reqwest-rustls")))
)]
#[cfg(any(feature = "reqwest-native", feature = "reqwest-rustls"))]
pub mod reqwest;
#[cfg_attr(docsrs, doc(cfg(feature = "surf")))]
#[cfg(feature = "surf")]
pub mod surf;

#[cfg(test)]
mod tests;
