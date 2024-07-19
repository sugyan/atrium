use super::super::decision::ModerationDecision;
use super::super::types::{LabelTarget, SubjectFeedGenerator};
use super::super::Moderator;

impl Moderator {
    pub(crate) fn decide_feed_generator(
        &self,
        subject: &SubjectFeedGenerator,
    ) -> ModerationDecision {
        let mut acc = ModerationDecision::new();
        acc.set_did(subject.creator.did.clone());
        acc.set_is_me(self.user_did.as_ref() == Some(&subject.creator.did));
        if let Some(labels) = &subject.labels {
            for label in labels {
                acc.add_label(LabelTarget::Content, label, self);
            }
        }
        ModerationDecision::merge(&[
            acc,
            self.decide_account(&subject.creator.clone().into()),
            self.decide_profile(&subject.creator.clone().into()),
        ])
    }
}
