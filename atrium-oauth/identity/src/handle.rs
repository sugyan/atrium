mod appview_resolver;
mod atproto_resolver;
mod dns_resolver;
#[cfg(feature = "doh-handle-resolver")]
mod doh_dns_txt_resolver;
mod well_known_resolver;

pub use self::appview_resolver::{AppViewHandleResolver, AppViewHandleResolverConfig};
pub use self::atproto_resolver::{AtprotoHandleResolver, AtprotoHandleResolverConfig};
pub use self::dns_resolver::DnsTxtResolver;
#[cfg(feature = "doh-handle-resolver")]
pub use self::doh_dns_txt_resolver::{DohDnsTxtResolver, DohDnsTxtResolverConfig};
pub use self::well_known_resolver::{WellKnownHandleResolver, WellKnownHandleResolverConfig};
use atrium_api::types::string::{Did, Handle};
use atrium_common::resolver::Resolver;

pub trait HandleResolver: Resolver<Input = Handle, Output = Did> {}
