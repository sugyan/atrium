use super::did_resolver::DidResolver;
use super::error::{Error, Result};
use super::HandleResolver;
use atrium_api::types::string::AtIdentifier;
use std::sync::Arc;

#[derive(Debug)]
pub struct ResolvedIdentity {
    pub did: String,
    pub pds: String,
}

pub struct IdentityResolver {
    did_resolver: Arc<dyn DidResolver + Send + Sync + 'static>,
    handle_resolver: Arc<dyn HandleResolver + Send + Sync + 'static>,
}

impl IdentityResolver {
    pub fn new(
        did_resolver: Arc<dyn DidResolver + Send + Sync + 'static>,
        handle_resolver: Arc<dyn HandleResolver + Send + Sync + 'static>,
    ) -> Self {
        // TODO: cached resolver?
        Self {
            did_resolver,
            handle_resolver,
        }
    }
    pub async fn resolve(&self, input: &str) -> Result<ResolvedIdentity> {
        let document = match input.parse().map_err(Error::AtIdentifier)? {
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
        Ok(ResolvedIdentity {
            did: document.id,
            pds: service,
        })
    }
}
