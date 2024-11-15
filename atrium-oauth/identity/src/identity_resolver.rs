use crate::error::{Error, Result};
use crate::{did::DidResolver, handle::HandleResolver};
use atrium_api::types::string::AtIdentifier;
use atrium_common::resolver::Resolver;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedIdentity {
    pub did: String,
    pub pds: String,
}

#[derive(Clone, Debug)]
pub struct IdentityResolverConfig<D, H> {
    pub did_resolver: D,
    pub handle_resolver: H,
}

pub struct IdentityResolver<D, H> {
    did_resolver: D,
    handle_resolver: H,
}

impl<D, H> IdentityResolver<D, H> {
    pub fn new(config: IdentityResolverConfig<D, H>) -> Self {
        Self { did_resolver: config.did_resolver, handle_resolver: config.handle_resolver }
    }
}

impl<D, H> Resolver for IdentityResolver<D, H>
where
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
    // Error: From<D::Error> + From<H::Error>,
{
    type Input = str;
    type Output = ResolvedIdentity;
    type Error = Error;

    async fn resolve(&self, input: &Self::Input) -> Result<Self::Output> {
        let document =
            match input.parse::<AtIdentifier>().map_err(|e| Error::AtIdentifier(e.to_string()))? {
                AtIdentifier::Did(did) => self.did_resolver.resolve(&did).await?,
                AtIdentifier::Handle(handle) => {
                    let did = self.handle_resolver.resolve(&handle).await?;
                    let document = self.did_resolver.resolve(&did).await?;
                    if let Some(aka) = &document.also_known_as {
                        if !aka.contains(&format!("at://{}", handle.as_str())) {
                            return Err(Error::DidDocument(format!(
                                "did document for `{}` does not include the handle `{}`",
                                did.as_str(),
                                handle.as_str()
                            )));
                        }
                    }
                    document
                }
            };
        let Some(service) = document.get_pds_endpoint() else {
            return Err(Error::DidDocument(format!(
                "no valid `AtprotoPersonalDataServer` service found in `{}`",
                document.id
            )));
        };
        Ok(ResolvedIdentity { did: document.id, pds: service })
    }
}
