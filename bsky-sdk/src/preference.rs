//! Preferences for Bluesky application.
use crate::moderation::ModerationPrefs;
use atrium_api::app::bsky::actor::defs::SavedFeed;
use atrium_api::types::Object;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A preference for a feed view.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FeedViewPreferenceData {
    pub hide_replies: bool,
    pub hide_replies_by_unfollowed: bool,
    pub hide_replies_by_like_count: i64,
    pub hide_reposts: bool,
    pub hide_quote_posts: bool,
}

impl Default for FeedViewPreferenceData {
    fn default() -> Self {
        Self {
            hide_replies: false,
            hide_replies_by_unfollowed: true,
            hide_replies_by_like_count: 0,
            hide_reposts: false,
            hide_quote_posts: false,
        }
    }
}

pub type FeedViewPreference = Object<FeedViewPreferenceData>;

/// A preference for a thread view.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ThreadViewPreferenceData {
    pub sort: String,
    pub prioritize_followed_users: bool,
}

impl ThreadViewPreferenceData {
    pub const SORT_OLDEST: &'static str = "oldest";
    pub const SORT_NEWEST: &'static str = "newest";
    pub const SORT_MOST_LIKES: &'static str = "most-likes";
    pub const SORT_RANDOM: &'static str = "random";
}

impl Default for ThreadViewPreferenceData {
    fn default() -> Self {
        Self { sort: Self::SORT_OLDEST.to_string(), prioritize_followed_users: true }
    }
}

pub type ThreadViewPreference = Object<ThreadViewPreferenceData>;

/// Preferences for Bluesky application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub saved_feeds: Vec<SavedFeed>,
    pub feed_view_prefs: HashMap<String, FeedViewPreference>,
    pub thread_view_prefs: ThreadViewPreference,
    pub moderation_prefs: ModerationPrefs,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            saved_feeds: Default::default(),
            feed_view_prefs: Default::default(),
            thread_view_prefs: ThreadViewPreferenceData::default().into(),
            moderation_prefs: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moderation::ModerationPrefsLabeler;
    use atrium_api::app::bsky::actor::get_preferences::Output;
    use serde_json::{from_str, to_string, Value};

    const XRPC_PREFERENCES_JSON: &str = r#"{
    "preferences": [
        {
            "$type": "app.bsky.actor.defs#savedFeedsPrefV2",
            "items": [
                {
                    "id": "3kt2b4tp6gs2a",
                    "pinned": true,
                    "type": "timeline",
                    "value": "following"
                }
            ]
        },
        {
            "$type": "app.bsky.actor.defs#threadViewPref",
            "sort": "oldest",
            "lab_treeViewEnabled": false
        },
        {
            "$type": "app.bsky.actor.defs#feedViewPref",
            "feed": "home",
            "hideRepliesByUnfollowed": false,
            "lab_mergeFeedEnabled": true
        }
    ]
}"#;

    #[test]
    fn xrpc_preferences_json() {
        let deserialized1 = from_str::<Output>(XRPC_PREFERENCES_JSON)
            .expect("deserializing preferences should succeed");
        assert_eq!(deserialized1.preferences.len(), 3);
        let serialized = to_string(&deserialized1).expect("serializing preferences should succeed");
        assert_eq!(
            serialized.replace(char::is_whitespace, ""),
            XRPC_PREFERENCES_JSON.replace(char::is_whitespace, "")
        );
        let deserialized2 =
            from_str::<Output>(&serialized).expect("deserializing preferences should succeed");
        assert_eq!(deserialized1, deserialized2);
    }

    #[test]
    fn sdk_preferences_json() {
        let preferences = Preferences {
            saved_feeds: Vec::new(),
            feed_view_prefs: HashMap::new(),
            thread_view_prefs: ThreadViewPreferenceData::default().into(),
            moderation_prefs: ModerationPrefs {
                labelers: vec![
                    ModerationPrefsLabeler::default(),
                    ModerationPrefsLabeler {
                        did: "did:fake:labeler.test".parse().expect("invalid did"),
                        labels: HashMap::new(),
                        is_default_labeler: false,
                    },
                ],
                ..Default::default()
            },
        };
        let serialized1 = to_string(&preferences).expect("serializing preferences should succeed");
        let deserialized = from_str::<Preferences>(&serialized1)
            .expect("deserializing preferences should succeed");
        assert_eq!(preferences, deserialized);
        let serialized2 = to_string(&deserialized).expect("serializing preferences should succeed");
        assert_eq!(
            from_str::<Value>(&serialized1).expect("deserializing to value should succeed"),
            from_str::<Value>(&serialized2).expect("deserializing to value should succeed"),
        );
    }
}
