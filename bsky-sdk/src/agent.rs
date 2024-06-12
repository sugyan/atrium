//! Implementation of [`BskyAgent`] and their builder.
mod builder;
pub mod config;

pub use self::builder::BskyAgentBuilder;
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
#[cfg(feature = "default-client")]
use atrium_xrpc_client::reqwest::ReqwestClient;
use ipld_core::serde::from_ipld;
use std::collections::HashMap;
use std::ops::Deref;

#[cfg(feature = "default-client")]
pub struct BskyAgent<T = ReqwestClient, S = MemorySessionStore>
where
    T: XrpcClient + Send + Sync,
    S: SessionStore + Send + Sync,
{
    inner: AtpAgent<S, T>,
}

#[cfg(not(feature = "default-client"))]
pub struct BskyAgent<T, S = MemorySessionStore>
where
    T: XrpcClient + Send + Sync,
    S: SessionStore + Send + Sync,
{
    inner: AtpAgent<S, T>,
}

#[cfg(feature = "default-client")]
impl BskyAgent {
    pub fn builder() -> BskyAgentBuilder<ReqwestClient, MemorySessionStore> {
        BskyAgentBuilder::default()
    }
}

impl<T, S> BskyAgent<T, S>
where
    T: XrpcClient + Send + Sync,
    S: SessionStore + Send + Sync,
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
                Union::Refs(PreferencesItem::AdultContentPref(p)) => {
                    prefs.moderation_prefs.adult_content_enabled = p.enabled;
                }
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

impl<T, S> Deref for BskyAgent<T, S>
where
    T: XrpcClient + Send + Sync,
    S: SessionStore + Send + Sync,
{
    type Target = AtpAgent<S, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
