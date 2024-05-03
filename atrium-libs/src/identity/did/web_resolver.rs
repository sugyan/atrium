use super::error::{Error, Result};
use super::{Fetch, Resolve};
use async_trait::async_trait;
use std::marker::PhantomData;
use url::{Host, Url};

#[derive(Debug, Default)]
pub struct DidWebResolver<T> {
    timeout: Option<u64>,
    _fetcher: PhantomData<T>,
}

impl<T> DidWebResolver<T> {
    pub fn new(timeout: Option<u64>) -> Self {
        Self {
            timeout,
            _fetcher: PhantomData,
        }
    }
}

#[async_trait]
impl<T> Resolve for DidWebResolver<T>
where
    T: Fetch + Send + Sync,
{
    async fn resolve_no_check(&self, did: &str) -> Result<Option<Vec<u8>>> {
        let parts = did.splitn(3, ':').collect::<Vec<_>>();
        if parts[2].is_empty() {
            return Err(Error::PoorlyFormattedDid(did.to_string()));
        }
        if parts[2].contains(':') {
            return Err(Error::UnsupportedDidWebPath(did.to_string()));
        }
        let mut url = Url::parse(&format!(
            "https://{}/.well-known/did.json",
            urlencoding::decode(parts[2])?
        ))?;
        if match url.host() {
            Some(Host::Domain(domain)) if domain == "localhost" => true,
            Some(Host::Ipv4(addr)) => addr.is_loopback(),
            Some(Host::Ipv6(addr)) => addr.is_loopback(),
            _ => false,
        } {
            url.set_scheme("http")
                .expect("failed to set scheme to http");
        }
        T::fetch(url.as_ref(), self.timeout)
            .await
            .map_err(Error::Fetch)
    }
}
