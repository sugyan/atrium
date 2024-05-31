pub mod config;

use self::config::Config;
use crate::error::Result;
use crate::moderation::util::interpret_label_value_definitions;
use crate::moderation::{ModerationPrefsLabeler, Moderator};
use crate::preference::Preferences;
use atrium_api::agent::store::MemorySessionStore;
use atrium_api::agent::{store::SessionStore, AtpAgent};
use atrium_api::app::bsky::actor::defs::{LabelersPref, PreferencesItem};
use atrium_api::types::Union;
use atrium_api::xrpc::XrpcClient;
use atrium_xrpc_client::reqwest::ReqwestClient;
use ipld_core::serde::from_ipld;
use std::collections::HashMap;
use std::ops::Deref;

pub struct BskyAgent<S = MemorySessionStore, T = ReqwestClient>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    inner: AtpAgent<S, T>,
}

impl BskyAgent {
    pub fn builder() -> BskyAgentBuilder<MemorySessionStore, ReqwestClient> {
        BskyAgentBuilder::default()
    }
}

impl<S, T> BskyAgent<S, T>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    pub async fn to_config(&self) -> Config {
        Config {
            endpoint: self.get_endpoint().await,
            session: self.get_session().await,
            labelers_header: self.get_labelers_header().await,
            proxy_header: self.get_proxy_header().await,
        }
    }
    pub async fn get_preferences(&self, enable_bsky_labeler: bool) -> Result<Preferences> {
        let mut prefs = Preferences::default();
        if enable_bsky_labeler {
            prefs
                .moderation_prefs
                .labelers
                .push(ModerationPrefsLabeler::default());
        }
        let mut label_prefs = Vec::new();
        for pref in self
            .api
            .app
            .bsky
            .actor
            .get_preferences(atrium_api::app::bsky::actor::get_preferences::Parameters {})
            .await?
            .preferences
        {
            match pref {
                Union::Refs(PreferencesItem::ContentLabelPref(p)) => {
                    label_prefs.push(p);
                }
                Union::Refs(PreferencesItem::MutedWordsPref(p)) => {
                    prefs.moderation_prefs.muted_words = p.items;
                }
                Union::Refs(PreferencesItem::HiddenPostsPref(p)) => {
                    prefs.moderation_prefs.hidden_posts = p.items;
                }
                Union::Unknown(u) => {
                    if u.r#type == "app.bsky.actor.defs#labelersPref" {
                        prefs.moderation_prefs.labelers.extend(
                            from_ipld::<LabelersPref>(u.data)?
                                .labelers
                                .into_iter()
                                .map(|item| ModerationPrefsLabeler {
                                    did: item.did,
                                    labels: HashMap::default(),
                                    is_default_labeler: false,
                                }),
                        );
                    }
                }
                _ => {
                    // TODO
                }
            }
        }
        for pref in label_prefs {
            if let Some(did) = pref.labeler_did {
                if let Some(l) = prefs
                    .moderation_prefs
                    .labelers
                    .iter_mut()
                    .find(|l| l.did == did)
                {
                    l.labels.insert(
                        pref.label,
                        pref.visibility.parse().expect("invalid visibility"),
                    );
                }
            } else {
                prefs.moderation_prefs.labels.insert(
                    pref.label,
                    pref.visibility.parse().expect("invalid visibility"),
                );
            }
        }
        Ok(prefs)
    }
    pub fn configure_labelers_from_preferences(&self, preferences: &Preferences) {
        self.configure_labelers_header(Some(
            preferences
                .moderation_prefs
                .labelers
                .iter()
                .map(|labeler| (labeler.did.clone(), labeler.is_default_labeler))
                .take(10)
                .collect(),
        ));
    }
    pub async fn moderator(&self, preferences: &Preferences) -> Result<Moderator> {
        let labelers = self
            .api
            .app
            .bsky
            .labeler
            .get_services(atrium_api::app::bsky::labeler::get_services::Parameters {
                detailed: Some(true),
                dids: preferences
                    .moderation_prefs
                    .labelers
                    .iter()
                    .map(|labeler| labeler.did.clone())
                    .collect(),
            })
            .await?
            .views;
        let mut label_defs = HashMap::with_capacity(labelers.len());
        for labeler in &labelers {
            let Union::Refs(atrium_api::app::bsky::labeler::get_services::OutputViewsItem::AppBskyLabelerDefsLabelerViewDetailed(labeler_view)) = labeler else {
                continue;
            };
            label_defs.insert(
                labeler_view.creator.did.clone(),
                interpret_label_value_definitions(labeler_view)?,
            );
        }
        Ok(Moderator::new(
            self.get_session().await.map(|s| s.did),
            preferences.moderation_prefs.clone(),
            label_defs,
        ))
    }
}

impl<S, T> Deref for BskyAgent<S, T>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    type Target = AtpAgent<S, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct BskyAgentBuilder<S, T>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    config: Config,
    store: S,
    client: T,
}

impl<S, T> BskyAgentBuilder<S, T>
where
    S: SessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
{
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }
    pub fn store<S0>(self, store: S0) -> BskyAgentBuilder<S0, T>
    where
        S0: SessionStore + Send + Sync,
    {
        BskyAgentBuilder {
            config: self.config,
            store,
            client: self.client,
        }
    }
    pub fn client<T0>(self, client: T0) -> BskyAgentBuilder<S, T0>
    where
        T0: XrpcClient + Send + Sync,
    {
        BskyAgentBuilder {
            config: self.config,
            store: self.store,
            client,
        }
    }
    pub async fn build(self) -> Result<BskyAgent<S, T>> {
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

impl Default for BskyAgentBuilder<MemorySessionStore, ReqwestClient> {
    fn default() -> Self {
        Self {
            config: Config::default(),
            client: ReqwestClient::new(Config::default().endpoint),
            store: MemorySessionStore::default(),
        }
    }
}
