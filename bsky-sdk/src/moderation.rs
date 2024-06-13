//! Moderation module for working with Bluesky's moderation.
pub mod decision;
mod error;
mod labels;
pub mod mutewords;
mod subjects;
mod types;
pub mod ui;
pub mod util;

use self::decision::ModerationDecision;
pub use self::error::{Error, Result};
pub use self::types::*;
use atrium_api::types::string::Did;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A moderator for the different kinds of content on the Bluesky network.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Moderator {
    user_did: Option<Did>,
    prefs: ModerationPrefs,
    label_defs: HashMap<Did, Vec<InterpretedLabelValueDefinition>>,
}

impl Moderator {
    /// Create a new moderator.
    pub fn new(
        user_did: Option<Did>,
        prefs: ModerationPrefs,
        label_defs: HashMap<Did, Vec<InterpretedLabelValueDefinition>>,
    ) -> Self {
        Self {
            user_did,
            prefs,
            label_defs,
        }
    }
    /// Calculate the moderation decision for an account profile.
    pub fn moderate_profile(&self, profile: &SubjectProfile) -> ModerationDecision {
        ModerationDecision::merge(&[self.decide_account(profile), self.decide_profile(profile)])
    }
    /// Calculate the moderation decision for a post.
    pub fn moderate_post(&self, post: &SubjectPost) -> ModerationDecision {
        self.decide_post(post)
    }
    /// Calculate the moderation decision for a notification.
    pub fn moderate_notification(&self) -> ModerationDecision {
        todo!()
    }
    /// Calculate the moderation decision for a feed generator.
    pub fn moderate_feed_generator(&self) -> ModerationDecision {
        todo!()
    }
    /// Calculate the moderation decision for a user list.
    pub fn moderate_user_list(&self) -> ModerationDecision {
        todo!()
    }
}

#[cfg(test)]
mod tests;
