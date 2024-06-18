use super::super::decision::ModerationDecision;
use super::super::types::{LabelTarget, SubjectNotification};
use super::super::Moderator;

impl Moderator {
    pub(crate) fn decide_notification(&self, subject: &SubjectNotification) -> ModerationDecision {
        let mut acc = ModerationDecision::new();
        acc.set_did(subject.author.did.clone());
        acc.set_is_me(self.user_did.as_ref() == Some(&subject.author.did));
        if let Some(labels) = &subject.labels {
            for label in labels {
                acc.add_label(LabelTarget::Content, label, self);
            }
        }
        ModerationDecision::merge(&[
            acc,
            self.decide_account(&subject.author.clone().into()),
            self.decide_profile(&subject.author.clone().into()),
        ])
    }
}
