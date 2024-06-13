use super::config::Config;
use super::BskyAgent;
use crate::error::Result;
use atrium_api::agent::store::MemorySessionStore;
use atrium_api::agent::{store::SessionStore, AtpAgent};
use atrium_api::xrpc::XrpcClient;
#[cfg(feature = "default-client")]
use atrium_xrpc_client::reqwest::ReqwestClient;

/// A builder for creating a [`BskyAgent`].
pub struct BskyAgentBuilder<T, S = MemorySessionStore>
where
    T: XrpcClient + Send + Sync,
    S: SessionStore + Send + Sync,
{
    config: Config,
    store: S,
    client: T,
}

impl<T> BskyAgentBuilder<T>
where
    T: XrpcClient + Send + Sync,
{
    /// Create a new builder with the given XRPC client.
    pub fn new(client: T) -> Self {
        Self {
            config: Config::default(),
            store: MemorySessionStore::default(),
            client,
        }
    }
}

impl<T, S> BskyAgentBuilder<T, S>
where
    T: XrpcClient + Send + Sync,
    S: SessionStore + Send + Sync,
{
    /// Set the configuration for the agent.
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }
    /// Set the session store for the agent.
    ///
    /// Returns a new builder with the session store set.
    pub fn store<S0>(self, store: S0) -> BskyAgentBuilder<T, S0>
    where
        S0: SessionStore + Send + Sync,
    {
        BskyAgentBuilder {
            config: self.config,
            store,
            client: self.client,
        }
    }
    /// Set the XRPC client for the agent.
    ///
    /// Returns a new builder with the XRPC client set.
    pub fn client<T0>(self, client: T0) -> BskyAgentBuilder<T0, S>
    where
        T0: XrpcClient + Send + Sync,
    {
        BskyAgentBuilder {
            config: self.config,
            store: self.store,
            client,
        }
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
        Ok(BskyAgent { inner: agent })
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "default-client")))]
#[cfg(feature = "default-client")]
impl Default for BskyAgentBuilder<ReqwestClient, MemorySessionStore> {
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
    use async_trait::async_trait;
    use atrium_api::agent::Session;

    fn session() -> Session {
        Session {
            access_jwt: String::new(),
            did: "did:fake:handle.test".parse().expect("invalid did"),
            did_doc: None,
            email: None,
            email_auth_factor: None,
            email_confirmed: None,
            handle: "handle.test".parse().expect("invalid handle"),
            refresh_jwt: String::new(),
        }
    }

    struct MockSessionStore;

    #[async_trait]
    impl SessionStore for MockSessionStore {
        async fn get_session(&self) -> Option<Session> {
            Some(session())
        }
        async fn set_session(&self, _: Session) {}
        async fn clear_session(&self) {}
    }

    #[cfg(feature = "default-client")]
    #[tokio::test]
    async fn default() -> Result<()> {
        // default build
        {
            let agent = BskyAgentBuilder::default().build().await?;
            assert_eq!(agent.get_endpoint().await, "https://bsky.social");
            assert_eq!(agent.get_session().await, None);
        }
        // with store
        {
            let agent = BskyAgentBuilder::default()
                .store(MockSessionStore)
                .build()
                .await?;
            assert_eq!(agent.get_endpoint().await, "https://bsky.social");
            assert_eq!(
                agent.get_session().await.map(|session| session.handle),
                Some("handle.test".parse().expect("invalid handle"))
            );
        }
        // with config
        {
            let agent = BskyAgentBuilder::default()
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
            let agent = BskyAgentBuilder::new(MockClient).build().await?;
            assert_eq!(agent.get_endpoint().await, "https://bsky.social");
        }
        // with store
        {
            let agent = BskyAgentBuilder::new(MockClient)
                .store(MockSessionStore)
                .build()
                .await?;
            assert_eq!(agent.get_endpoint().await, "https://bsky.social");
            assert_eq!(
                agent.get_session().await.map(|session| session.handle),
                Some("handle.test".parse().expect("invalid handle"))
            );
        }
        // with config
        {
            let agent = BskyAgentBuilder::new(MockClient)
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
