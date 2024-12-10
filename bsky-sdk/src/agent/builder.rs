use super::config::Config;
use super::BskyAgent;
use crate::error::Result;
use atrium_api::agent::atp_agent::{
    store::{AtpSessionStore, MemorySessionStore},
    AtpAgent,
};
use atrium_api::xrpc::XrpcClient;
#[cfg(feature = "default-client")]
use atrium_xrpc_client::reqwest::ReqwestClient;
use std::sync::Arc;

/// A builder for creating a [`BskyAtpAgent`].
pub struct BskyAtpAgentBuilder<T, S = MemorySessionStore>
where
    T: XrpcClient + Send + Sync,
    S: AtpSessionStore + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    config: Config,
    store: S,
    client: T,
}

impl<T> BskyAtpAgentBuilder<T>
where
    T: XrpcClient + Send + Sync,
{
    /// Create a new builder with the given XRPC client.
    pub fn new(client: T) -> Self {
        Self { config: Config::default(), store: MemorySessionStore::default(), client }
    }
}

impl<T, S> BskyAtpAgentBuilder<T, S>
where
    T: XrpcClient + Send + Sync,
    S: AtpSessionStore + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    /// Set the configuration for the agent.
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }
    /// Set the session store for the agent.
    ///
    /// Returns a new builder with the session store set.
    pub fn store<S0>(self, store: S0) -> BskyAtpAgentBuilder<T, S0>
    where
        S0: AtpSessionStore + Send + Sync,
        S0::Error: std::error::Error + Send + Sync + 'static,
    {
        BskyAtpAgentBuilder { config: self.config, store, client: self.client }
    }
    /// Set the XRPC client for the agent.
    ///
    /// Returns a new builder with the XRPC client set.
    pub fn client<T0>(self, client: T0) -> BskyAtpAgentBuilder<T0, S>
    where
        T0: XrpcClient + Send + Sync,
    {
        BskyAtpAgentBuilder { config: self.config, store: self.store, client }
    }
    pub async fn build(self) -> Result<BskyAgent<T, S>> {
        let agent = AtpAgent::new(self.client, self.store);
        agent.configure_endpoint(self.config.endpoint);
        if let Some(session) = self.config.session {
            agent.resume_session(session).await?;
        }
        if let Some(labelers) = self.config.labelers_header {
            agent.configure_labelers_header(Some(
                labelers
                    .iter()
                    .filter_map(|did| {
                        let (did, redact) = match did.split_once(';') {
                            Some((did, params)) if params.trim() == "redact" => (did, true),
                            None => (did.as_str(), false),
                            _ => return None,
                        };
                        did.parse().ok().map(|did| (did, redact))
                    })
                    .collect(),
            ));
        }
        if let Some(proxy) = self.config.proxy_header {
            if let Some((did, service_type)) = proxy.split_once('#') {
                if let Ok(did) = did.parse() {
                    agent.configure_proxy_header(did, service_type);
                }
            }
        }
        Ok(BskyAgent { inner: Arc::new(agent) })
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "default-client")))]
#[cfg(feature = "default-client")]
impl Default for BskyAtpAgentBuilder<ReqwestClient, MemorySessionStore> {
    /// Create a new builder with the default client and session store.
    ///
    /// Default client is [`ReqwestClient`] and default session store is [`MemorySessionStore`].
    fn default() -> Self {
        Self::new(ReqwestClient::new(Config::default().endpoint))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::tests::MockSessionStore;

    #[cfg(feature = "default-client")]
    #[tokio::test]
    async fn default() -> Result<()> {
        // default build
        {
            let agent = BskyAtpAgentBuilder::default().build().await?;
            assert_eq!(agent.get_endpoint().await, "https://bsky.social");
            assert_eq!(agent.get_session().await, None);
        }
        // with store
        {
            let agent = BskyAtpAgentBuilder::default().store(MockSessionStore).build().await?;
            assert_eq!(agent.get_endpoint().await, "https://bsky.social");
            assert_eq!(
                agent.get_session().await.map(|session| session.data.handle),
                Some("handle.test".parse().expect("invalid handle"))
            );
        }
        // with config
        {
            let agent = BskyAtpAgentBuilder::default()
                .config(Config {
                    endpoint: "https://example.com".to_string(),
                    ..Default::default()
                })
                .build()
                .await?;
            assert_eq!(agent.get_endpoint().await, "https://example.com");
            assert_eq!(agent.get_session().await, None);
        }
        Ok(())
    }

    #[cfg(not(feature = "default-client"))]
    #[tokio::test]
    async fn custom() -> Result<()> {
        use crate::tests::MockClient;

        // default build
        {
            let agent = BskyAtpAgentBuilder::new(MockClient).build().await?;
            assert_eq!(agent.get_endpoint().await, "https://bsky.social");
        }
        // with store
        {
            let agent =
                BskyAtpAgentBuilder::new(MockClient).store(MockSessionStore).build().await?;
            assert_eq!(agent.get_endpoint().await, "https://bsky.social");
            assert_eq!(
                agent.get_session().await.map(|session| session.data.handle),
                Some("handle.test".parse().expect("invalid handle"))
            );
        }
        // with config
        {
            let agent = BskyAtpAgentBuilder::new(MockClient)
                .config(Config {
                    endpoint: "https://example.com".to_string(),
                    ..Default::default()
                })
                .build()
                .await?;
            assert_eq!(agent.get_endpoint().await, "https://example.com");
            assert_eq!(agent.get_session().await, None);
        }
        Ok(())
    }
}
