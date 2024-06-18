//! Preferences for Bluesky application.
use crate::moderation::ModerationPrefs;
use atrium_api::app::bsky::actor::defs::SavedFeed;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A preference for a feed view.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedViewPreference {
    pub hide_replies: bool,
    pub hide_replies_by_unfollowed: bool,
    pub hide_replies_by_like_count: i64,
    pub hide_reposts: bool,
    pub hide_quote_posts: bool,
}

impl Default for FeedViewPreference {
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

/// Preferences for Bluesky application.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub saved_feeds: Vec<SavedFeed>,
    pub feed_view_prefs: HashMap<String, FeedViewPreference>,
    pub moderation_prefs: ModerationPrefs,
}

#[cfg(test)]
mod tests {
    use atrium_api::app::bsky::actor::get_preferences::Output;
    use serde_json::{from_str, to_string};

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
            "$type": "app.bsky.actor.defs#feedViewPref",
            "feed": "home",
            "hideRepliesByUnfollowed": false,
            "lab_mergeFeedEnabled": true
        }
    ]
}"#;

    #[test]
    fn xrpc_preferences_json() {
        let deserialized = from_str::<Output>(XRPC_PREFERENCES_JSON)
            .expect("deserializing preferences should succeed");
        assert_eq!(deserialized.preferences.len(), 2);
        let serialized = to_string(&deserialized).expect("serializing preferences should succeed");
        assert_eq!(
            serialized.replace(char::is_whitespace, ""),
            XRPC_PREFERENCES_JSON.replace(char::is_whitespace, "")
        );
    }
}
