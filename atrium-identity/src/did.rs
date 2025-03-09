mod common_resolver;
mod plc_resolver;
mod web_resolver;

pub use self::common_resolver::{CommonDidResolver, CommonDidResolverConfig};
pub use self::plc_resolver::DEFAULT_PLC_DIRECTORY_URL;
use crate::Error;
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;
use atrium_common::resolver::Resolver;

pub trait DidResolver: Resolver<Input = Did, Output = DidDocument, Error = Error> {}
