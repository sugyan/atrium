use super::{assert_ui, label, post_view, profile_view_basic};
use super::{ExpectedBehaviors, ResultFlag};
use crate::moderation::decision::DecisionContext;
use crate::moderation::error::Result;
use crate::moderation::types::*;
use crate::moderation::util::interpret_label_value_definition;
use crate::moderation::Moderator;
use atrium_api::com::atproto::label::defs::LabelValueDefinitionData;
use std::collections::HashMap;

#[derive(Debug)]
struct Scenario {
    blurs: LabelValueDefinitionBlurs,
    severity: LabelValueDefinitionSeverity,
    account: ExpectedBehaviors,
    profile: ExpectedBehaviors,
    post: ExpectedBehaviors,
}

impl Scenario {
    fn run(&self) {
        let moderator = self.moderator().expect("failed to create moderator");
        // account
        {
            let result = moderator.moderate_profile(
                &profile_view_basic(
                    "bob.test",
                    Some("Bob"),
                    Some(vec![label("did:web:labeler.test", "did:web:bob.test", "custom")]),
                )
                .into(),
            );
            for context in DecisionContext::ALL {
                assert_ui(&result, self.account.expected_for(context), context);
            }
        }
        // profile
        {
            let result = moderator.moderate_profile(
                &profile_view_basic(
                    "bob.test",
                    Some("Bob"),
                    Some(vec![label(
                        "did:web:labeler.test",
                        "at://did:web:bob.test/app.bsky.actor.profile/self",
                        "custom",
                    )]),
                )
                .into(),
            );
            for context in DecisionContext::ALL {
                assert_ui(&result, self.profile.expected_for(context), context);
            }
        }
        // post
        {
            let result = moderator.moderate_post(&post_view(
                &profile_view_basic("bob.test", Some("Bob"), None),
                "Hello",
                Some(vec![label(
                    "did:web:labeler.test",
                    "at://did:web:bob.test/app.bsky.feed.post/fake",
                    "custom",
                )]),
            ));
            for context in DecisionContext::ALL {
                assert_ui(&result, self.post.expected_for(context), context);
            }
        }
    }
    fn moderator(&self) -> Result<Moderator> {
        Ok(Moderator::new(
            Some("did:web:alice.test".parse().expect("invalid did")),
            ModerationPrefs {
                adult_content_enabled: true,
                labels: HashMap::new(),
                labelers: vec![ModerationPrefsLabeler {
                    did: "did:web:labeler.test".parse().expect("invalid did"),
                    labels: HashMap::from_iter([(String::from("custom"), LabelPreference::Hide)]),
                    is_default_labeler: false,
                }],
                muted_words: Vec::new(),
                hidden_posts: Vec::new(),
            },
            HashMap::from_iter([(
                "did:web:labeler.test".parse().expect("invalid did"),
                vec![interpret_label_value_definition(
                    &LabelValueDefinitionData {
                        adult_only: None,
                        blurs: self.blurs.as_ref().to_string(),
                        default_setting: Some(LabelPreference::Warn.as_ref().to_string()),
                        identifier: String::from("custom"),
                        locales: Vec::new(),
                        severity: self.severity.as_ref().to_string(),
                    }
                    .into(),
                    Some("did:web:labeler.test".parse().expect("invalid did")),
                )?],
            )]),
        ))
    }
}

#[test]
fn moderation_custom_labels() {
    use ResultFlag::*;
    let scenarios = [
        Scenario {
            blurs: LabelValueDefinitionBlurs::Content,
            severity: LabelValueDefinitionSeverity::Alert,
            account: ExpectedBehaviors {
                profile_list: vec![Filter, Alert],
                profile_view: vec![Alert],
                content_list: vec![Filter, Blur],
                content_view: vec![Alert],
                ..Default::default()
            },
            profile: ExpectedBehaviors {
                profile_list: vec![Alert],
                profile_view: vec![Alert],
                ..Default::default()
            },
            post: ExpectedBehaviors {
                content_list: vec![Filter, Blur],
                content_view: vec![Alert],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::Content,
            severity: LabelValueDefinitionSeverity::Inform,
            account: ExpectedBehaviors {
                profile_list: vec![Filter, Inform],
                profile_view: vec![Inform],
                content_list: vec![Filter, Blur],
                content_view: vec![Inform],
                ..Default::default()
            },
            profile: ExpectedBehaviors {
                profile_list: vec![Inform],
                profile_view: vec![Inform],
                ..Default::default()
            },
            post: ExpectedBehaviors {
                content_list: vec![Filter, Blur],
                content_view: vec![Inform],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::Content,
            severity: LabelValueDefinitionSeverity::None,
            account: ExpectedBehaviors {
                profile_list: vec![Filter],
                content_list: vec![Filter, Blur],
                ..Default::default()
            },
            profile: ExpectedBehaviors { ..Default::default() },
            post: ExpectedBehaviors { content_list: vec![Filter, Blur], ..Default::default() },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::Media,
            severity: LabelValueDefinitionSeverity::Alert,
            account: ExpectedBehaviors {
                profile_list: vec![Filter, Alert],
                profile_view: vec![Alert],
                avatar: vec![Blur],
                banner: vec![Blur],
                content_list: vec![Filter],
                ..Default::default()
            },
            profile: ExpectedBehaviors {
                profile_list: vec![Alert],
                profile_view: vec![Alert],
                avatar: vec![Blur],
                banner: vec![Blur],
                ..Default::default()
            },
            post: ExpectedBehaviors {
                content_list: vec![Filter],
                content_media: vec![Blur],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::Media,
            severity: LabelValueDefinitionSeverity::Inform,
            account: ExpectedBehaviors {
                profile_list: vec![Filter, Inform],
                profile_view: vec![Inform],
                avatar: vec![Blur],
                banner: vec![Blur],
                content_list: vec![Filter],
                ..Default::default()
            },
            profile: ExpectedBehaviors {
                profile_list: vec![Inform],
                profile_view: vec![Inform],
                avatar: vec![Blur],
                banner: vec![Blur],
                ..Default::default()
            },
            post: ExpectedBehaviors {
                content_list: vec![Filter],
                content_media: vec![Blur],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::Media,
            severity: LabelValueDefinitionSeverity::None,
            account: ExpectedBehaviors {
                profile_list: vec![Filter],
                avatar: vec![Blur],
                banner: vec![Blur],
                content_list: vec![Filter],
                ..Default::default()
            },
            profile: ExpectedBehaviors {
                avatar: vec![Blur],
                banner: vec![Blur],
                ..Default::default()
            },
            post: ExpectedBehaviors {
                content_list: vec![Filter],
                content_media: vec![Blur],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::None,
            severity: LabelValueDefinitionSeverity::Alert,
            account: ExpectedBehaviors {
                profile_list: vec![Filter, Alert],
                profile_view: vec![Alert],
                content_list: vec![Filter, Alert],
                content_view: vec![Alert],
                ..Default::default()
            },
            profile: ExpectedBehaviors {
                profile_list: vec![Alert],
                profile_view: vec![Alert],
                ..Default::default()
            },
            post: ExpectedBehaviors {
                content_list: vec![Filter, Alert],
                content_view: vec![Alert],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::None,
            severity: LabelValueDefinitionSeverity::Inform,
            account: ExpectedBehaviors {
                profile_list: vec![Filter, Inform],
                profile_view: vec![Inform],
                content_list: vec![Filter, Inform],
                content_view: vec![Inform],
                ..Default::default()
            },
            profile: ExpectedBehaviors {
                profile_list: vec![Inform],
                profile_view: vec![Inform],
                ..Default::default()
            },
            post: ExpectedBehaviors {
                content_list: vec![Filter, Inform],
                content_view: vec![Inform],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::None,
            severity: LabelValueDefinitionSeverity::None,
            account: ExpectedBehaviors {
                profile_list: vec![Filter],
                content_list: vec![Filter],
                ..Default::default()
            },
            profile: ExpectedBehaviors { ..Default::default() },
            post: ExpectedBehaviors { content_list: vec![Filter], ..Default::default() },
        },
    ];
    for scenario in scenarios {
        scenario.run();
    }
}
