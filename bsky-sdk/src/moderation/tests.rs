mod behaviors;
mod custom_labels;
mod mutewords;
mod quoteposts;

use crate::moderation::decision::{DecisionContext, ModerationDecision};
use crate::moderation::types::*;
use crate::moderation::util::interpret_label_value_definition;
use crate::moderation::Moderator;
use crate::tests::FAKE_CID;
use atrium_api::app::bsky::actor::defs::{ProfileViewBasic, ProfileViewBasicData};
use atrium_api::app::bsky::feed::defs::{PostView, PostViewData};
use atrium_api::com::atproto::label::defs::{Label, LabelData, LabelValueDefinitionData};
use atrium_api::records::{KnownRecord, Record};
use atrium_api::types::string::Datetime;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResultFlag {
    Filter,
    Blur,
    Alert,
    Inform,
    NoOverride,
}

#[derive(Debug, Default)]
struct ExpectedBehaviors {
    profile_list: Vec<ResultFlag>,
    profile_view: Vec<ResultFlag>,
    avatar: Vec<ResultFlag>,
    banner: Vec<ResultFlag>,
    display_name: Vec<ResultFlag>,
    content_list: Vec<ResultFlag>,
    content_view: Vec<ResultFlag>,
    content_media: Vec<ResultFlag>,
}

impl ExpectedBehaviors {
    fn expected_for(&self, context: DecisionContext) -> &Vec<ResultFlag> {
        match context {
            DecisionContext::ProfileList => &self.profile_list,
            DecisionContext::ProfileView => &self.profile_view,
            DecisionContext::Avatar => &self.avatar,
            DecisionContext::Banner => &self.banner,
            DecisionContext::DisplayName => &self.display_name,
            DecisionContext::ContentList => &self.content_list,
            DecisionContext::ContentView => &self.content_view,
            DecisionContext::ContentMedia => &self.content_media,
        }
    }
}

fn profile_view_basic(
    handle: &str,
    display_name: Option<&str>,
    labels: Option<Vec<Label>>,
) -> ProfileViewBasic {
    ProfileViewBasicData {
        associated: None,
        avatar: None,
        created_at: None,
        did: format!("did:web:{handle}").parse().expect("invalid did"),
        display_name: display_name.map(String::from),
        handle: handle.parse().expect("invalid handle"),
        labels,
        viewer: None,
    }
    .into()
}

fn post_view(author: &ProfileViewBasic, text: &str, labels: Option<Vec<Label>>) -> PostView {
    PostViewData {
        author: author.clone(),
        cid: FAKE_CID.parse().expect("invalid cid"),
        embed: None,
        indexed_at: Datetime::now(),
        labels,
        like_count: None,
        record: Record::Known(KnownRecord::AppBskyFeedPost(Box::new(
            atrium_api::app::bsky::feed::post::RecordData {
                created_at: Datetime::now(),
                embed: None,
                entities: None,
                facets: None,
                labels: None,
                langs: None,
                reply: None,
                tags: None,
                text: text.into(),
            }
            .into(),
        ))),
        reply_count: None,
        repost_count: None,
        threadgate: None,
        uri: format!("at://{}/app.bsky.feed.post/fake", author.did.as_ref()),
        viewer: None,
    }
    .into()
}

fn label(src: &str, uri: &str, val: &str) -> Label {
    LabelData {
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
    .into()
}

fn assert_ui(decision: &ModerationDecision, expected: &[ResultFlag], context: DecisionContext) {
    let ui = decision.ui(context);
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
            expected.contains(&ResultFlag::Inform),
            "inform should be {} for context {context:?}",
            !ui.inform()
        );
        assert_eq!(
            ui.alert(),
            expected.contains(&ResultFlag::Alert),
            "alert should be {} for context {context:?}",
            !ui.alert()
        );
        assert_eq!(
            ui.blur(),
            expected.contains(&ResultFlag::Blur),
            "blur should be {} for context {context:?}",
            !ui.blur()
        );
        assert_eq!(
            ui.filter(),
            expected.contains(&ResultFlag::Filter),
            "filter should be {} for context {context:?}",
            !ui.filter()
        );
        assert_eq!(
            ui.no_override,
            expected.contains(&ResultFlag::NoOverride),
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
        let moderator = Moderator::new(
            Some("did:web:alice.test".parse().expect("invalid did")),
            ModerationPrefs {
                adult_content_enabled: true,
                labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Hide)]),
                ..Default::default()
            },
            HashMap::new(),
        );
        let result = moderator.moderate_profile(&profile);
        assert_ui(&result, &[ResultFlag::Blur], DecisionContext::Avatar)
    }
    // porn (ignore)
    {
        let moderator = Moderator::new(
            Some("did:web:alice.test".parse().expect("invalid did")),
            ModerationPrefs {
                adult_content_enabled: true,
                labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Ignore)]),
                ..Default::default()
            },
            HashMap::new(),
        );
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
        let moderator = Moderator::new(
            Some("did:web:alice.test".parse().expect("invalid did")),
            ModerationPrefs {
                adult_content_enabled: true,
                labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Hide)]),
                ..Default::default()
            },
            HashMap::new(),
        );
        let result = moderator.moderate_profile(&profile);
        for context in DecisionContext::ALL {
            assert_ui(&result, &[], context);
        }
    }
    // porn (label group disabled)
    {
        let moderator = Moderator::new(
            Some("did:web:alice.test".parse().expect("invalid did")),
            ModerationPrefs {
                adult_content_enabled: true,
                labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Hide)]),
                labelers: vec![ModerationPrefsLabeler {
                    did: "did:web:labeler.test".parse().expect("invalid did"),
                    labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Ignore)]),
                    is_default_labeler: false,
                }],
                ..Default::default()
            },
            HashMap::new(),
        );
        let result = moderator.moderate_profile(&profile);
        for context in DecisionContext::ALL {
            assert_ui(&result, &[], context);
        }
    }
}

#[test]
fn prioritize_filters_and_blurs() {
    let moderator = Moderator::new(
        Some("did:web:alice.test".parse().expect("invalid did")),
        ModerationPrefs {
            adult_content_enabled: true,
            labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Hide)]),
            labelers: vec![ModerationPrefsLabeler {
                did: "did:web:labeler.test".parse().expect("invalid did"),
                labels: HashMap::new(),
                is_default_labeler: false,
            }],
            ..Default::default()
        },
        HashMap::new(),
    );
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
    let moderator = Moderator::new(
        Some("did:web:alice.test".parse().expect("invalid did")),
        ModerationPrefs {
            adult_content_enabled: true,
            labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Warn)]),
            labelers: vec![ModerationPrefsLabeler {
                did: "did:web:labeler.test".parse().expect("invalid did"),
                labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Warn)]),
                is_default_labeler: false,
            }],
            ..Default::default()
        },
        HashMap::from_iter([(
            "did:web:labeler.test".parse().expect("invalid did"),
            vec![interpret_label_value_definition(
                &LabelValueDefinitionData {
                    identifier: String::from("porn"),
                    default_setting: Some(String::from("warn")),
                    severity: String::from("inform"),
                    blurs: String::from("none"),
                    adult_only: None,
                    locales: Vec::new(),
                }
                .into(),
                Some("did:web:labeler.test".parse().expect("invalid did")),
            )
            .expect("invalid label value definition")],
        )]),
    );
    let result = moderator.moderate_post(&post_view(
        &profile_view_basic("bob.test", Some("Bob"), None),
        "Hello",
        Some(vec![label(
            "did:web:labeler.test",
            "at://did:web:bob.test/app.bsky.post/fake",
            "porn",
        )]),
    ));
    for context in DecisionContext::ALL {
        let expected = match context {
            DecisionContext::ContentList => vec![ResultFlag::Inform],
            DecisionContext::ContentView => vec![ResultFlag::Inform],
            _ => vec![],
        };
        assert_ui(&result, &expected, context);
    }
}

#[test]
fn does_not_override_imperative_labels() {
    let moderator = Moderator::new(
        Some("did:web:alice.test".parse().expect("invalid did")),
        ModerationPrefs {
            adult_content_enabled: true,
            labels: HashMap::new(),
            labelers: vec![ModerationPrefsLabeler {
                did: "did:web:labeler.test".parse().expect("invalid did"),
                labels: HashMap::new(),
                is_default_labeler: false,
            }],
            ..Default::default()
        },
        HashMap::from_iter([(
            "did:web:labeler.test".parse().expect("invalid did"),
            vec![interpret_label_value_definition(
                &LabelValueDefinitionData {
                    identifier: String::from("!hide"),
                    default_setting: Some(String::from("warn")),
                    severity: String::from("inform"),
                    blurs: String::from("none"),
                    adult_only: None,
                    locales: Vec::new(),
                }
                .into(),
                Some("did:web:labeler.test".parse().expect("invalid did")),
            )
            .expect("invalid label value definition")],
        )]),
    );
    let result = moderator.moderate_post(&post_view(
        &profile_view_basic("bob.test", Some("Bob"), None),
        "Hello",
        Some(vec![label(
            "did:web:labeler.test",
            "at://did:web:bob.test/app.bsky.post/fake",
            "!hide",
        )]),
    ));
    for context in DecisionContext::ALL {
        let expected = match context {
            DecisionContext::ContentList => {
                vec![ResultFlag::Filter, ResultFlag::Blur, ResultFlag::NoOverride]
            }
            DecisionContext::ContentView => vec![ResultFlag::Blur, ResultFlag::NoOverride],
            _ => vec![],
        };
        assert_ui(&result, &expected, context);
    }
}

#[test]
fn ignore_invalid_label_value_names() {
    let moderator = Moderator::new(
        Some("did:web:alice.test".parse().expect("invalid did")),
        ModerationPrefs {
            adult_content_enabled: true,
            labels: HashMap::new(),
            labelers: vec![ModerationPrefsLabeler {
                did: "did:web:labeler.test".parse().expect("invalid did"),
                labels: HashMap::from_iter([
                    (String::from("BadLabel"), LabelPreference::Hide),
                    (String::from("bad/label"), LabelPreference::Hide),
                ]),
                is_default_labeler: false,
            }],
            ..Default::default()
        },
        HashMap::from_iter([(
            "did:web:labeler.test".parse().expect("invalid did"),
            vec![
                interpret_label_value_definition(
                    &LabelValueDefinitionData {
                        identifier: String::from("BadLabel"),
                        default_setting: Some(String::from("warn")),
                        severity: String::from("inform"),
                        blurs: String::from("content"),
                        adult_only: None,
                        locales: Vec::new(),
                    }
                    .into(),
                    Some("did:web:labeler.test".parse().expect("invalid did")),
                )
                .expect("invalid label value definition"),
                interpret_label_value_definition(
                    &LabelValueDefinitionData {
                        identifier: String::from("bad/label"),
                        default_setting: Some(String::from("warn")),
                        severity: String::from("inform"),
                        blurs: String::from("content"),
                        adult_only: None,
                        locales: Vec::new(),
                    }
                    .into(),
                    Some("did:web:labeler.test".parse().expect("invalid did")),
                )
                .expect("invalid label value definition"),
            ],
        )]),
    );
    let result = moderator.moderate_post(&post_view(
        &profile_view_basic("bob.test", Some("Bob"), None),
        "Hello",
        Some(vec![
            label(
                "did:web:labeler.test",
                "at://did:web:bob.test/app.bsky.post/fake",
                "BadLabel",
            ),
            label(
                "did:web:labeler.test",
                "at://did:web:bob.test/app.bsky.post/fake",
                "bad/label",
            ),
        ]),
    ));
    for context in DecisionContext::ALL {
        assert_ui(&result, &[], context);
    }
}

#[test]
fn custom_labels_with_default_settings() {
    let moderator = Moderator::new(
        Some("did:web:alice.test".parse().expect("invalid did")),
        ModerationPrefs {
            adult_content_enabled: true,
            labels: HashMap::new(),
            labelers: vec![ModerationPrefsLabeler {
                did: "did:web:labeler.test".parse().expect("invalid did"),
                labels: HashMap::new(),
                is_default_labeler: false,
            }],
            ..Default::default()
        },
        HashMap::from_iter([(
            "did:web:labeler.test".parse().expect("invalid did"),
            vec![
                interpret_label_value_definition(
                    &LabelValueDefinitionData {
                        identifier: String::from("default-hide"),
                        default_setting: Some(String::from("hide")),
                        severity: String::from("inform"),
                        blurs: String::from("content"),
                        adult_only: None,
                        locales: Vec::new(),
                    }
                    .into(),
                    Some("did:web:labeler.test".parse().expect("invalid did")),
                )
                .expect("invalid label value definition"),
                interpret_label_value_definition(
                    &LabelValueDefinitionData {
                        identifier: String::from("default-warn"),
                        default_setting: Some(String::from("warn")),
                        severity: String::from("inform"),
                        blurs: String::from("content"),
                        adult_only: None,
                        locales: Vec::new(),
                    }
                    .into(),
                    Some("did:web:labeler.test".parse().expect("invalid did")),
                )
                .expect("invalid label value definition"),
                interpret_label_value_definition(
                    &LabelValueDefinitionData {
                        identifier: String::from("default-ignore"),
                        default_setting: Some(String::from("ignore")),
                        severity: String::from("inform"),
                        blurs: String::from("content"),
                        adult_only: None,
                        locales: Vec::new(),
                    }
                    .into(),
                    Some("did:web:labeler.test".parse().expect("invalid did")),
                )
                .expect("invalid label value definition"),
            ],
        )]),
    );
    let author = profile_view_basic("bob.test", Some("Bob"), None);
    {
        let result = moderator.moderate_post(&post_view(
            &author,
            "Hello",
            Some(vec![label(
                "did:web:labeler.test",
                "at://did:web:bob.test/app.bsky.post/fake",
                "default-hide",
            )]),
        ));
        for context in DecisionContext::ALL {
            let expected = match context {
                DecisionContext::ContentList => vec![ResultFlag::Filter, ResultFlag::Blur],
                DecisionContext::ContentView => vec![ResultFlag::Inform],
                _ => vec![],
            };
            assert_ui(&result, &expected, context);
        }
    }
    {
        let result = moderator.moderate_post(&post_view(
            &author,
            "Hello",
            Some(vec![label(
                "did:web:labeler.test",
                "at://did:web:bob.test/app.bsky.post/fake",
                "default-warn",
            )]),
        ));
        for context in DecisionContext::ALL {
            let expected = match context {
                DecisionContext::ContentList => vec![ResultFlag::Blur],
                DecisionContext::ContentView => vec![ResultFlag::Inform],
                _ => vec![],
            };
            assert_ui(&result, &expected, context);
        }
    }
    {
        let result = moderator.moderate_post(&post_view(
            &author,
            "Hello",
            Some(vec![label(
                "did:web:labeler.test",
                "at://did:web:bob.test/app.bsky.post/fake",
                "default-ignore",
            )]),
        ));
        for context in DecisionContext::ALL {
            assert_ui(&result, &[], context)
        }
    }
}

#[test]
fn custom_labels_require_adult_content_enabled() {
    let moderator = Moderator::new(
        Some("did:web:alice.test".parse().expect("invalid did")),
        ModerationPrefs {
            adult_content_enabled: false,
            labels: HashMap::from_iter([(String::from("adult"), LabelPreference::Ignore)]),
            labelers: vec![ModerationPrefsLabeler {
                did: "did:web:labeler.test".parse().expect("invalid did"),
                labels: HashMap::from_iter([(String::from("adult"), LabelPreference::Ignore)]),
                is_default_labeler: false,
            }],
            ..Default::default()
        },
        HashMap::from_iter([(
            "did:web:labeler.test".parse().expect("invalid did"),
            vec![interpret_label_value_definition(
                &LabelValueDefinitionData {
                    identifier: String::from("adult"),
                    default_setting: Some(String::from("hide")),
                    severity: String::from("inform"),
                    blurs: String::from("content"),
                    adult_only: Some(true),
                    locales: Vec::new(),
                }
                .into(),
                Some("did:web:labeler.test".parse().expect("invalid did")),
            )
            .expect("invalid label value definition")],
        )]),
    );
    let result = moderator.moderate_post(&post_view(
        &profile_view_basic("bob.test", Some("Bob"), None),
        "Hello",
        Some(vec![label(
            "did:web:labeler.test",
            "at://did:web:bob.test/app.bsky.post/fake",
            "adult",
        )]),
    ));
    for context in DecisionContext::ALL {
        let expected = match context {
            DecisionContext::ContentList => {
                vec![ResultFlag::Filter, ResultFlag::Blur, ResultFlag::NoOverride]
            }
            DecisionContext::ContentView => vec![ResultFlag::Blur, ResultFlag::NoOverride],
            _ => vec![],
        };
        assert_ui(&result, &expected, context);
    }
}

#[test]
fn adult_content_disabled_forces_hide() {
    let moderator = Moderator::new(
        Some("did:web:alice.test".parse().expect("invalid did")),
        ModerationPrefs {
            adult_content_enabled: false,
            labels: HashMap::from_iter([(String::from("porn"), LabelPreference::Ignore)]),
            labelers: vec![ModerationPrefsLabeler {
                did: "did:web:labeler.test".parse().expect("invalid did"),
                labels: HashMap::new(),
                is_default_labeler: false,
            }],
            ..Default::default()
        },
        HashMap::new(),
    );
    let result = moderator.moderate_post(&post_view(
        &profile_view_basic("bob.test", Some("Bob"), None),
        "Hello",
        Some(vec![label(
            "did:web:labeler.test",
            "at://did:web:bob.test/app.bsky.post/fake",
            "porn",
        )]),
    ));
    for context in DecisionContext::ALL {
        let expected = match context {
            DecisionContext::ContentList => vec![ResultFlag::Filter],
            DecisionContext::ContentMedia => vec![ResultFlag::Blur, ResultFlag::NoOverride],
            _ => vec![],
        };
        assert_ui(&result, &expected, context);
    }
}
