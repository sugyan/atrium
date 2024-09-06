mod base_resolver;
mod common_resolver;
mod plc_resolver;
mod web_resolver;

use super::Resolver;
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;
pub use common_resolver::{CommonDidResolver, CommonDidResolverConfig};
pub(crate) use plc_resolver::DEFAULT_PLC_DIRECTORY_URL;

pub trait DidResolver: Resolver<Input = Did, Output = DidDocument> {}
