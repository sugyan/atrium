use super::super::decision::ModerationDecision;
use super::super::types::{LabelTarget, SubjectUserList};
use super::super::Moderator;
use atrium_api::types::string::Did;

impl Moderator {
    pub(crate) fn decide_user_list(&self, subject: &SubjectUserList) -> ModerationDecision {
        let mut acc = ModerationDecision::new();
        match subject {
            SubjectUserList::ListView(list_view) => {
                acc.set_did(list_view.creator.did.clone());
                acc.set_is_me(self.user_did.as_ref() == Some(&list_view.creator.did));
                if let Some(labels) = &list_view.labels {
                    for label in labels {
                        acc.add_label(LabelTarget::Content, label, self);
                    }
                }
                ModerationDecision::merge(&[
                    acc,
                    self.decide_account(&list_view.creator.clone().into()),
                    self.decide_profile(&list_view.creator.clone().into()),
                ])
            }
            SubjectUserList::ListViewBasic(list_view_basic) => {
                let did = list_view_basic
                    .uri
                    .strip_prefix("at://")
                    .expect("invalid at-uri")
                    .split_once('/')
                    .expect("invalid at-uri")
                    .0
                    .parse::<Did>()
                    .expect("invalid did");
                acc.set_did(did.clone());
                acc.set_is_me(self.user_did.as_ref() == Some(&did));
                if let Some(labels) = &list_view_basic.labels {
                    for label in labels {
                        acc.add_label(LabelTarget::Content, label, self);
                    }
                }
                acc
            }
        }
    }
}
