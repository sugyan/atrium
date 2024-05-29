pub mod decision;
mod labels;
mod types;
pub mod ui;

use self::decision::{LabelTarget, ModerationDecision};
pub use self::types::*;
use atrium_api::types::{string::Did, Union};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Moderator {
    user_did: Option<Did>,
    prefs: ModerationPrefs,
    label_defs: Option<HashMap<String, Vec<InterpretedLabelValueDefinition>>>,
}

impl Moderator {
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
        // TODO: muted?
        // TODO: blocked?
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
mod tests {
    use super::decision::DecisionContext;
    use super::*;
    use atrium_api::app::bsky::actor::defs::ProfileViewBasic;
    use atrium_api::app::bsky::feed::defs::PostView;
    use atrium_api::com::atproto::label::defs::Label;
    use atrium_api::records::{KnownRecord, Record};
    use atrium_api::types::string::Datetime;

    const FAKE_CID: &str = "bafyreiclp443lavogvhj3d2ob2cxbfuscni2k5jk7bebjzg7khl3esabwq";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ModerationTestResultFlag {
        Filter,
        Blur,
        Alert,
        Inform,
        NoOverride,
    }

    fn profile_view_basic(
        handle: &str,
        display_name: Option<&str>,
        labels: Option<Vec<Label>>,
    ) -> ProfileViewBasic {
        ProfileViewBasic {
            associated: None,
            avatar: None,
            did: format!("did:web:{handle}").parse().expect("invalid did"),
            display_name: display_name.map(String::from),
            handle: handle.parse().expect("invalid handle"),
            labels,
            viewer: None,
        }
    }

    fn post_view(author: &ProfileViewBasic, text: &str, labels: Option<Vec<Label>>) -> PostView {
        PostView {
            author: author.clone(),
            cid: FAKE_CID.parse().expect("invalid cid"),
            embed: None,
            indexed_at: Datetime::now(),
            labels,
            like_count: None,
            record: Record::Known(KnownRecord::AppBskyFeedPost(Box::new(
                atrium_api::app::bsky::feed::post::Record {
                    created_at: Datetime::now(),
                    embed: None,
                    entities: None,
                    facets: None,
                    labels: None,
                    langs: None,
                    reply: None,
                    tags: None,
                    text: text.into(),
                },
            ))),
            reply_count: None,
            repost_count: None,
            threadgate: None,
            uri: format!("at://{}/app.bsky.feed.post/fake", author.did.as_ref()),
            viewer: None,
        }
    }

    fn label(src: &str, uri: &str, val: &str) -> Label {
        Label {
            cid: None,
            cts: Datetime::now(),
            exp: None,
            neg: None,
            sig: None,
            src: src.parse().expect("invalid did"),
            uri: uri.into(),
            val: val.into(),
            ver: None,
        }
    }

    fn interpreted_label_value_definition(
        identifier: &str,
        default_setting: LabelPreference,
        severity: &str,
        blurs: &str,
    ) -> InterpretedLabelValueDefinition {
        let flags = vec![LabelValueDefinitionFlag::NoSelf];
        let alert_or_inform = match severity {
            "alert" => BehaviorValue::Alert,
            "inform" => BehaviorValue::Inform,
            _ => unreachable!(),
        };
        let mut behaviors = InterpretedLabelValueDefinitionBehaviors::default();
        match blurs {
            "content" => {
                todo!()
            }
            "media" => {
                todo!()
            }
            "none" => {
                // target=account, blurs=none
                behaviors.account.profile_list = Some(alert_or_inform.try_into().unwrap());
                behaviors.account.profile_view = Some(alert_or_inform.try_into().unwrap());
                behaviors.account.content_list = Some(alert_or_inform.try_into().unwrap());
                behaviors.account.content_view = Some(alert_or_inform.try_into().unwrap());
                // target=profile, blurs=none
                behaviors.profile.profile_list = Some(alert_or_inform.try_into().unwrap());
                behaviors.profile.profile_view = Some(alert_or_inform.try_into().unwrap());
                // target=content, blurs=none
                behaviors.content.content_list = Some(alert_or_inform.try_into().unwrap());
                behaviors.content.content_view = Some(alert_or_inform.try_into().unwrap());
            }
            _ => unreachable!(),
        }
        InterpretedLabelValueDefinition {
            identifier: identifier.into(),
            default_setting,
            flags,
            behaviors,
        }
    }

    fn assert_ui(
        decision: &ModerationDecision,
        expected: &[ModerationTestResultFlag],
        context: DecisionContext,
    ) {
        let ui = decision.ui(context);
        println!("{:?}", ui.inform());
        println!("{:?}", ui.blur());
        if expected.is_empty() {
            assert!(
                !ui.inform(),
                "inform should be a no-op for context {context:?}"
            );
            assert!(
                !ui.alert(),
                "alert should be a no-op for context {context:?}"
            );
            assert!(!ui.blur(), "blur should be a no-op for context {context:?}");
            assert!(
                !ui.filter(),
                "filter should be a no-op for context {context:?}"
            );
            assert!(
                !ui.no_override,
                "no_override should be a no-op for context {context:?}"
            );
        } else {
            assert_eq!(
                ui.inform(),
                expected.contains(&ModerationTestResultFlag::Inform),
                "inform should be {} for context {context:?}",
                !ui.inform()
            );
            assert_eq!(
                ui.alert(),
                expected.contains(&ModerationTestResultFlag::Alert),
                "alert should be {} for context {context:?}",
                !ui.alert()
            );
            assert_eq!(
                ui.blur(),
                expected.contains(&ModerationTestResultFlag::Blur),
                "blur should be {} for context {context:?}",
                !ui.blur()
            );
            assert_eq!(
                ui.filter(),
                expected.contains(&ModerationTestResultFlag::Filter),
                "filter should be {} for context {context:?}",
                !ui.filter()
            );
            assert_eq!(
                ui.no_override,
                expected.contains(&ModerationTestResultFlag::NoOverride),
                "no_override should be {} for context {context:?}",
                !ui.no_override
            );
        }
    }

    #[test]
    fn self_label_global() {
        let profile = SubjectProfile::from(profile_view_basic(
            "bob.test",
            Some("Bob"),
            Some(vec![label(
                "did:web:bob.test",
                "at://did:web:bob.test/app.bsky.actor.profile/self",
                "porn",
            )]),
        ));
        // porn (hide)
        {
            let moderator = Moderator {
                user_did: Some("did:web:alice.test".parse().expect("invalid did")),
                prefs: ModerationPrefs {
                    adult_content_enabled: true,
                    labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Hide)]),
                    ..Default::default()
                },
                label_defs: None,
            };
            let result = moderator.moderate_profile(&profile);
            assert_ui(
                &result,
                &[ModerationTestResultFlag::Blur],
                DecisionContext::Avatar,
            )
        }
        // porn (ignore)
        {
            let moderator = Moderator {
                user_did: Some("did:web:alice.test".parse().expect("invalid did")),
                prefs: ModerationPrefs {
                    adult_content_enabled: true,
                    labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Ignore)]),
                    ..Default::default()
                },
                label_defs: None,
            };
            let result = moderator.moderate_profile(&profile);
            assert_ui(&result, &[], DecisionContext::Avatar)
        }
    }

    #[test]
    fn unsubscribed_or_ignore_labels() {
        let profile = SubjectProfile::from(profile_view_basic(
            "bob.test",
            Some("Bob"),
            Some(vec![label(
                "did:web:labeler.test",
                "at://did:web:bob.test/app.bsky.actor.profile/self",
                "porn",
            )]),
        ));
        // porn (moderator disabled)
        {
            let moderator = Moderator {
                user_did: Some("did:web:alice.test".parse().expect("invalid did")),
                prefs: ModerationPrefs {
                    adult_content_enabled: true,
                    labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Hide)]),
                    ..Default::default()
                },
                label_defs: None,
            };
            let result = moderator.moderate_profile(&profile);
            for context in DecisionContext::ALL {
                assert_ui(&result, &[], context);
            }
        }
        // porn (label group disabled)
        {
            let moderator = Moderator {
                user_did: Some("did:web:alice.test".parse().expect("invalid did")),
                prefs: ModerationPrefs {
                    adult_content_enabled: true,
                    labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Hide)]),
                    labelers: vec![ModerationPrefsLabeler {
                        did: "did:web:labeler.test".parse().expect("invalid did"),
                        labels: HashMap::from_iter([(
                            String::from("porn"),
                            LabelPreference::Ignore,
                        )]),
                        is_default_labeler: false,
                    }],
                },
                label_defs: None,
            };
            let result = moderator.moderate_profile(&profile);
            for context in DecisionContext::ALL {
                assert_ui(&result, &[], context);
            }
        }
    }

    #[test]
    fn prioritize_filters_and_blurs() {
        let moderator = Moderator {
            user_did: Some("did:web:alice.test".parse().expect("invalid did")),
            prefs: ModerationPrefs {
                adult_content_enabled: true,
                labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Hide)]),
                labelers: vec![ModerationPrefsLabeler {
                    did: "did:web:labeler.test".parse().expect("invalid did"),
                    labels: HashMap::new(),
                    is_default_labeler: false,
                }],
            },
            label_defs: None,
        };
        let result = moderator.moderate_post(&post_view(
            &profile_view_basic("bob.test", Some("Bob"), None),
            "Hello",
            Some(vec![
                label(
                    "did:web:labeler.test",
                    "at://did:web:bob.test/app.bsky.post/fake",
                    "porn",
                ),
                label(
                    "did:web:labeler.test",
                    "at://did:web:bob.test/app.bsky.post/fake",
                    "!hide",
                ),
            ]),
        ));
        for (cause, expected_val) in [
            (&result.ui(DecisionContext::ContentList).filters[0], "!hide"),
            (&result.ui(DecisionContext::ContentList).filters[1], "porn"),
            (&result.ui(DecisionContext::ContentList).blurs[0], "!hide"),
            (&result.ui(DecisionContext::ContentMedia).blurs[0], "porn"),
        ] {
            if let ModerationCause::Label(label) = cause {
                assert_eq!(label.label.val, expected_val, "unexpected label value");
            } else {
                panic!("unexpected cause: {cause:?}");
            }
        }
    }

    #[test]
    fn prioritize_custom_labels() {
        let moderator = Moderator {
            user_did: Some("did:web:alice.test".parse().expect("invalid did")),
            prefs: ModerationPrefs {
                adult_content_enabled: true,
                labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Warn)]),
                labelers: vec![ModerationPrefsLabeler {
                    did: "did:web:labeler.test".parse().expect("invalid did"),
                    labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Warn)]),
                    is_default_labeler: false,
                }],
            },
            label_defs: Some(HashMap::from_iter([(
                String::from("did:web:labeler.test"),
                vec![interpreted_label_value_definition(
                    "porn",
                    LabelPreference::Warn,
                    "inform",
                    "none",
                )],
            )])),
        };
        let result = moderator.moderate_post(&post_view(
            &profile_view_basic("bob.test", Some("Bob"), None),
            "Hello",
            Some(vec![label(
                "did:web:labeler.test",
                "at://did:web:bob.test/app.bsky.post/fake",
                "porn",
            )]),
        ));
        assert_ui(&result, &[], DecisionContext::ProfileList);
        assert_ui(&result, &[], DecisionContext::ProfileView);
        assert_ui(&result, &[], DecisionContext::Avatar);
        assert_ui(&result, &[], DecisionContext::Banner);
        assert_ui(&result, &[], DecisionContext::DisplayName);
        assert_ui(
            &result,
            &[ModerationTestResultFlag::Inform],
            DecisionContext::ContentList,
        );
        assert_ui(
            &result,
            &[ModerationTestResultFlag::Inform],
            DecisionContext::ContentView,
        );
        assert_ui(&result, &[], DecisionContext::ContentMedia);
    }

    #[test]
    fn does_not_override_imperative_labels() {
        let moderator = Moderator {
            user_did: Some("did:web:alice.test".parse().expect("invalid did")),
            prefs: ModerationPrefs {
                adult_content_enabled: true,
                labels: HashMap::new(),
                labelers: vec![ModerationPrefsLabeler {
                    did: "did:web:labeler.test".parse().expect("invalid did"),
                    labels: HashMap::new(),
                    is_default_labeler: false,
                }],
            },
            label_defs: Some(HashMap::from_iter([(
                String::from("did:web:labeler.test"),
                vec![interpreted_label_value_definition(
                    "!hide",
                    LabelPreference::Warn,
                    "inform",
                    "none",
                )],
            )])),
        };
        let result = moderator.moderate_post(&post_view(
            &profile_view_basic("bob.test", Some("Bob"), None),
            "Hello",
            Some(vec![label(
                "did:web:labeler.test",
                "at://did:web:bob.test/app.bsky.post/fake",
                "!hide",
            )]),
        ));
        assert_ui(&result, &[], DecisionContext::ProfileList);
        assert_ui(&result, &[], DecisionContext::ProfileView);
        assert_ui(&result, &[], DecisionContext::Avatar);
        assert_ui(&result, &[], DecisionContext::Banner);
        assert_ui(&result, &[], DecisionContext::DisplayName);
        assert_ui(
            &result,
            &[
                ModerationTestResultFlag::Filter,
                ModerationTestResultFlag::Blur,
                ModerationTestResultFlag::NoOverride,
            ],
            DecisionContext::ContentList,
        );
        assert_ui(
            &result,
            &[
                ModerationTestResultFlag::Blur,
                ModerationTestResultFlag::NoOverride,
            ],
            DecisionContext::ContentView,
        );
        assert_ui(&result, &[], DecisionContext::ContentMedia);
    }
}
