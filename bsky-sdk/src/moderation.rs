pub mod decision;
mod labels;
mod types;
pub mod ui;
pub mod util;

use self::decision::ModerationDecision;
pub use self::types::*;
use atrium_api::types::{string::Did, Union};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Moderator {
    user_did: Option<Did>,
    prefs: ModerationPrefs,
    label_defs: HashMap<Did, Vec<InterpretedLabelValueDefinition>>,
}

impl Moderator {
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
    pub fn moderate_profile(&self, profile: &SubjectProfile) -> ModerationDecision {
        ModerationDecision::merge(&[
            self.account_decision(profile),
            self.profile_decision(profile),
        ])
    }
    pub fn moderate_post(&self, post: &SubjectPost) -> ModerationDecision {
        self.post_decision(post)
    }
    fn account_decision(&self, subject: &SubjectProfile) -> ModerationDecision {
        let mut acc = ModerationDecision::new();
        acc.set_did(subject.did().clone());
        acc.set_is_me(self.user_did.as_ref() == Some(subject.did()));
        if let Some(viewer) = subject.viewer() {
            if viewer.muted.unwrap_or_default() {
                if let Some(list_view) = &viewer.muted_by_list {
                    acc.add_muted_by_list(list_view);
                } else {
                    acc.add_muted();
                }
            }
            if viewer.blocking.is_some() {
                if let Some(list_view) = &viewer.blocking_by_list {
                    acc.add_blocking_by_list(list_view);
                } else {
                    acc.add_blocking();
                }
            }
            if viewer.blocked_by.unwrap_or_default() {
                acc.add_blocked_by();
            }
        }
        if let Some(labels) = subject.labels() {
            for label in labels.iter().filter(|l| {
                !l.uri.ends_with("/app.bsky.actor.profile/self") || l.val == "!no-unauthenticated"
            }) {
                acc.add_label(LabelTarget::Account, label, self);
            }
        }
        acc
    }
    fn profile_decision(&self, subject: &SubjectProfile) -> ModerationDecision {
        let mut acc = ModerationDecision::new();
        acc.set_did(subject.did().clone());
        acc.set_is_me(self.user_did.as_ref() == Some(subject.did()));
        if let Some(labels) = subject.labels() {
            for label in labels
                .iter()
                .filter(|l| l.uri.ends_with("/app.bsky.actor.profile/self"))
            {
                acc.add_label(LabelTarget::Profile, label, self);
            }
        }
        acc
    }
    fn post_decision(&self, subject: &SubjectPost) -> ModerationDecision {
        let mut acc = ModerationDecision::new();
        acc.set_did(subject.author.did.clone());
        acc.set_is_me(self.user_did.as_ref() == Some(&subject.author.did));
        if let Some(labels) = &subject.labels {
            for label in labels {
                acc.add_label(LabelTarget::Content, label, self);
            }
        }
        // TODO: hidden?
        // TODO: muted words?

        let embed_acc = Option::<ModerationDecision>::None;
        if let Some(Union::Refs(embed)) = &subject.embed {
            todo!()
        }

        let mut decisions = vec![acc];
        if let Some(mut embed_acc) = embed_acc {
            embed_acc.downgrade();
            decisions.push(embed_acc);
        }
        decisions.extend([
            self.account_decision(&subject.author.clone().into()),
            self.profile_decision(&subject.author.clone().into()),
        ]);
        ModerationDecision::merge(&decisions)
    }
}

#[cfg(test)]
mod tests;
