use super::super::decision::ModerationDecision;
use super::super::types::{LabelTarget, SubjectProfile};
use super::super::Moderator;

impl Moderator {
    pub(crate) fn decide_profile(&self, subject: &SubjectProfile) -> ModerationDecision {
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
}
