use super::super::decision::ModerationDecision;
use super::super::types::{LabelTarget, SubjectProfile};
use super::super::Moderator;

impl Moderator {
    pub fn decide_account(&self, subject: &SubjectProfile) -> ModerationDecision {
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
}
