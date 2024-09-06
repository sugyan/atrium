pub mod did;
mod error;
pub mod handle;
mod identity_resolver;

pub use self::error::{Error, Result};
use async_trait::async_trait;
pub use identity_resolver::{
    DidResolverConfig, HandleResolverConfig, IdentityResolver, IdentityResolverConfig,
    ResolvedIdentity,
};

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Resolver {
    type Input: ?Sized;
    type Output;

    async fn resolve(&self, input: &Self::Input) -> Result<Self::Output>;
}
