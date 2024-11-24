//! Implementation of [`BskyAgent`] and their builders.
mod builder;
pub mod config;

pub use self::builder::BskyAtpAgentBuilder;
use self::config::Config;
use crate::error::Result;
use crate::moderation::util::interpret_label_value_definitions;
use crate::moderation::{ModerationPrefsLabeler, Moderator};
use crate::preference::{FeedViewPreferenceData, Preferences, ThreadViewPreferenceData};
use atrium_api::agent::atp_agent::{AtpAgent, AtpSession};
use atrium_api::app::bsky::actor::defs::PreferencesItem;
use atrium_api::types::{Object, Union};
use atrium_api::xrpc::XrpcClient;
use atrium_common::store::memory::MemoryStore;
use atrium_common::store::Store;
#[cfg(feature = "default-client")]
use atrium_xrpc_client::reqwest::ReqwestClient;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

/// A Bluesky agent.
///
/// This agent is a wrapper around the [`Agent`](atrium_api::agent::Agent) that provides additional functionality for working with Bluesky.
/// For creating an instance of this agent, use the [`BskyAtpAgentBuilder`].
///
/// # Example
///
/// ```
/// use bsky_sdk::BskyAgent;
///
/// #[tokio::main]
/// async fn main() {
///    let agent = BskyAgent::builder().build().await.expect("failed to build agent");
/// }
/// ```

#[cfg(feature = "default-client")]
#[derive(Clone)]
pub struct BskyAgent<T = ReqwestClient, S = MemoryStore<(), AtpSession>>
where
    T: XrpcClient + Send + Sync,
    S: Store<(), AtpSession> + Send + Sync,
    S::Error: Send + Sync + 'static,
{
    inner: Arc<AtpAgent<S, T>>,
}

#[cfg(not(feature = "default-client"))]
pub struct BskyAgent<T, S = MemoryStore<(), AtpSession>>
where
    T: XrpcClient + Send + Sync,
    S: Store<(), AtpSession> + Send + Sync,
    S::Error: Send + Sync + 'static,
{
    inner: Arc<AtpAgent<S, T>>,
}

#[cfg_attr(docsrs, doc(cfg(feature = "default-client")))]
#[cfg(feature = "default-client")]
impl BskyAgent {
    /// Create a new [`BskyAtpAgentBuilder`] with the default client and session store.
    pub fn builder() -> BskyAtpAgentBuilder<ReqwestClient, MemoryStore<(), AtpSession>> {
        BskyAtpAgentBuilder::default()
    }
}

impl<T, S> BskyAgent<T, S>
where
    T: XrpcClient + Send + Sync,
    S: Store<(), AtpSession> + Send + Sync,
    S::Error: Send + Sync + 'static,
{
    /// Get the agent's current state as a [`Config`].
    pub async fn to_config(&self) -> Config {
        Config {
            endpoint: self.get_endpoint().await,
            session: self.get_session().await,
            labelers_header: self.get_labelers_header().await,
            proxy_header: self.get_proxy_header().await,
        }
    }
    /// Get the logged-in user's [`Preferences`].
    ///
    /// This implementation does not perform migration of `SavedFeedsPref` to V2.
    ///
    /// # Arguments
    ///
    /// `enable_bsky_labeler` - If `true`, the [Bluesky's moderation labeler](atrium_api::agent::bluesky::BSKY_LABELER_DID) will be included in the moderation preferences.
    pub async fn get_preferences(&self, enable_bsky_labeler: bool) -> Result<Preferences> {
        let mut prefs = Preferences::default();
        if enable_bsky_labeler {
            prefs.moderation_prefs.labelers.push(ModerationPrefsLabeler::default());
        }
        let mut label_prefs = Vec::new();
        for pref in self
            .api
            .app
            .bsky
            .actor
            .get_preferences(
                atrium_api::app::bsky::actor::get_preferences::ParametersData {}.into(),
            )
            .await?
            .data
            .preferences
        {
            match pref {
                Union::Refs(PreferencesItem::AdultContentPref(p)) => {
                    prefs.moderation_prefs.adult_content_enabled = p.enabled;
                }
                Union::Refs(PreferencesItem::ContentLabelPref(p)) => {
                    label_prefs.push(p);
                }
                Union::Refs(PreferencesItem::SavedFeedsPrefV2(p)) => {
                    prefs.saved_feeds = p.data.items;
                }
                Union::Refs(PreferencesItem::FeedViewPref(p)) => {
                    let mut pref = FeedViewPreferenceData::default();
                    if let Some(v) = p.hide_replies {
                        pref.hide_replies = v;
                    }
                    if let Some(v) = p.hide_replies_by_unfollowed {
                        pref.hide_replies_by_unfollowed = v;
                    }
                    if let Some(v) = p.hide_replies_by_like_count {
                        pref.hide_replies_by_like_count = v;
                    }
                    if let Some(v) = p.hide_reposts {
                        pref.hide_reposts = v;
                    }
                    if let Some(v) = p.hide_quote_posts {
                        pref.hide_quote_posts = v;
                    }
                    prefs.feed_view_prefs.insert(
                        p.data.feed,
                        Object {
                            data: pref,
                            extra_data: p.extra_data, // pass through extra data
                        },
                    );
                }
                Union::Refs(PreferencesItem::ThreadViewPref(p)) => {
                    let mut pref = ThreadViewPreferenceData::default();
                    if let Some(v) = &p.sort {
                        pref.sort = v.clone();
                    }
                    if let Some(v) = p.prioritize_followed_users {
                        pref.prioritize_followed_users = v;
                    }
                    prefs.thread_view_prefs = Object {
                        data: pref,
                        extra_data: p.extra_data, // pass through extra data
                    };
                }
                Union::Refs(PreferencesItem::MutedWordsPref(p)) => {
                    prefs.moderation_prefs.muted_words = p.data.items;
                }
                Union::Refs(PreferencesItem::HiddenPostsPref(p)) => {
                    prefs.moderation_prefs.hidden_posts = p.data.items;
                }
                Union::Refs(PreferencesItem::LabelersPref(p)) => {
                    prefs.moderation_prefs.labelers.extend(p.data.labelers.into_iter().map(
                        |item| ModerationPrefsLabeler {
                            did: item.data.did,
                            labels: HashMap::default(),
                            is_default_labeler: false,
                        },
                    ));
                }
                _ => {
                    // TODO
                }
            }
        }
        for pref in label_prefs {
            if let Some(did) = pref.data.labeler_did {
                if let Some(l) = prefs.moderation_prefs.labelers.iter_mut().find(|l| l.did == did) {
                    l.labels.insert(
                        pref.data.label,
                        pref.data.visibility.parse().expect("invalid visibility"),
                    );
                }
            } else {
                prefs.moderation_prefs.labels.insert(
                    pref.data.label,
                    pref.data.visibility.parse().expect("invalid visibility"),
                );
            }
        }
        Ok(prefs)
    }
    /// Configure the labelers header.
    ///
    /// Read labelers preferences from the provided [`Preferences`] and set the labelers header up to 10 labelers.
    ///
    /// See details: [https://docs.bsky.app/docs/advanced-guides/moderation#labeler-subscriptions](https://docs.bsky.app/docs/advanced-guides/moderation#labeler-subscriptions)
    pub fn configure_labelers_from_preferences(&self, preferences: &Preferences) {
        self.configure_labelers_header(Some(
            preferences
                .moderation_prefs
                .labelers
                .iter()
                .map(|labeler| (labeler.did.clone(), labeler.is_default_labeler))
                .collect(),
        ));
    }
    /// Make a [`Moderator`] instance with the provided [`Preferences`].
    pub async fn moderator(&self, preferences: &Preferences) -> Result<Moderator> {
        let views = if preferences.moderation_prefs.labelers.is_empty() {
            Vec::new()
        } else {
            self.api
                .app
                .bsky
                .labeler
                .get_services(
                    atrium_api::app::bsky::labeler::get_services::ParametersData {
                        detailed: Some(true),
                        dids: preferences
                            .moderation_prefs
                            .labelers
                            .iter()
                            .map(|labeler| labeler.did.clone())
                            .collect(),
                    }
                    .into(),
                )
                .await?
                .data
                .views
        };
        let mut label_defs = HashMap::with_capacity(views.len());
        for labeler in &views {
            let Union::Refs(atrium_api::app::bsky::labeler::get_services::OutputViewsItem::AppBskyLabelerDefsLabelerViewDetailed(labeler_view)) = labeler else {
                continue;
            };
            label_defs.insert(
                labeler_view.creator.did.clone(),
                interpret_label_value_definitions(labeler_view)?,
            );
        }
        Ok(Moderator::new(
            self.get_session().await.map(|s| s.data.did),
            preferences.moderation_prefs.clone(),
            label_defs,
        ))
    }
}

impl<T, S> Deref for BskyAgent<T, S>
where
    T: XrpcClient + Send + Sync,
    S: Store<(), AtpSession> + Send + Sync,
    S::Error: Send + Sync + 'static,
{
    type Target = AtpAgent<S, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atrium_api::agent::atp_agent::AtpSession;

    #[derive(Clone)]
    struct NoopStore;

    impl Store<(), AtpSession> for NoopStore {
        type Error = std::convert::Infallible;

        async fn get(&self, _key: &()) -> core::result::Result<Option<AtpSession>, Self::Error> {
            unimplemented!()
        }
        async fn set(&self, _key: (), _value: AtpSession) -> core::result::Result<(), Self::Error> {
            unimplemented!()
        }
        async fn del(&self, _key: &()) -> core::result::Result<(), Self::Error> {
            unimplemented!()
        }
        async fn clear(&self) -> core::result::Result<(), Self::Error> {
            unimplemented!()
        }
    }

    #[cfg(feature = "default-client")]
    #[tokio::test]
    async fn clone_agent() {
        let agent =
            BskyAgent::builder().store(NoopStore).build().await.expect("failed to build agent");
        let cloned = agent.clone();

        agent.configure_endpoint(String::from("https://example.com"));
        assert_eq!(cloned.get_endpoint().await, "https://example.com");
    }
}
