use super::{assert_ui, label, post_view, profile_view_basic};
use super::{ExpectedBehaviors, ResultFlag, FAKE_CID};
use crate::moderation::decision::DecisionContext;
use crate::moderation::error::Result;
use crate::moderation::types::*;
use crate::moderation::util::interpret_label_value_definition;
use crate::moderation::Moderator;
use atrium_api::app::bsky::actor::defs::ProfileViewBasic;
use atrium_api::app::bsky::embed::record::{ViewData, ViewRecordData, ViewRecordRefs};
use atrium_api::app::bsky::feed::defs::{PostView, PostViewEmbedRefs};
use atrium_api::com::atproto::label::defs::{Label, LabelValueDefinitionData};
use atrium_api::types::string::Datetime;
use atrium_api::types::{TryIntoUnknown, Union};
use std::collections::HashMap;

fn embed_record_view(
    author: &ProfileViewBasic,
    record: &atrium_api::app::bsky::feed::post::Record,
    labels: Option<Vec<Label>>,
) -> Union<PostViewEmbedRefs> {
    Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordView(Box::new(
        ViewData {
            record: Union::Refs(ViewRecordRefs::ViewRecord(Box::new(
                ViewRecordData {
                    author: author.clone(),
                    cid: FAKE_CID.parse().expect("invalid cid"),
                    embeds: None,
                    indexed_at: Datetime::now(),
                    labels,
                    like_count: None,
                    quote_count: None,
                    reply_count: None,
                    repost_count: None,
                    uri: format!("at://{}/app.bsky.feed.post/fake", author.did.as_ref()),
                    value: record
                        .try_into_unknown()
                        .expect("failed to convert record to unknown"),
                }
                .into(),
            ))),
        }
        .into(),
    )))
}

fn quoted_post(profile_labels: Option<Vec<Label>>, post_labels: Option<Vec<Label>>) -> PostView {
    let mut quoted = post_view(
        &profile_view_basic("bob.test", Some("Bob"), None),
        "Hello",
        None,
    );
    quoted.embed = Some(embed_record_view(
        &profile_view_basic("carla.test", Some("Carla"), profile_labels),
        &atrium_api::app::bsky::feed::post::RecordData {
            created_at: Datetime::now(),
            embed: None,
            entities: None,
            facets: None,
            labels: None,
            langs: Some(vec!["en".parse().expect("invalid lang")]),
            reply: None,
            tags: None,
            text: String::from("Quoted post text"),
        }
        .into(),
        post_labels,
    ));
    quoted
}

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
            let result = moderator.moderate_post(&quoted_post(
                Some(vec![label(
                    "did:web:labeler.test",
                    "did:web:carla.test",
                    "custom",
                )]),
                None,
            ));
            for context in DecisionContext::ALL {
                assert_ui(&result, self.account.expected_for(context), context);
            }
        }
        // profile
        {
            let result = moderator.moderate_post(&quoted_post(
                Some(vec![label(
                    "did:web:labeler.test",
                    "at://did:web:carla.test/app.bsky.actor.profile/self",
                    "custom",
                )]),
                None,
            ));
            for context in DecisionContext::ALL {
                assert_ui(&result, self.profile.expected_for(context), context);
            }
        }
        // post
        {
            let result = moderator.moderate_post(&quoted_post(
                None,
                Some(vec![label(
                    "did:web:labeler.test",
                    "at://did:web:carla.test/app.bsky.feed.post/fake",
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
fn moderation_quoteposts() {
    use ResultFlag::*;
    let scenarios = [
        Scenario {
            blurs: LabelValueDefinitionBlurs::Content,
            severity: LabelValueDefinitionSeverity::Alert,
            account: ExpectedBehaviors {
                profile_list: vec![Filter],
                content_list: vec![Filter],
                ..Default::default()
            },
            profile: ExpectedBehaviors::default(),
            post: ExpectedBehaviors {
                content_list: vec![Filter],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::Content,
            severity: LabelValueDefinitionSeverity::Inform,
            account: ExpectedBehaviors {
                profile_list: vec![Filter],
                content_list: vec![Filter],
                ..Default::default()
            },
            profile: ExpectedBehaviors::default(),
            post: ExpectedBehaviors {
                content_list: vec![Filter],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::Content,
            severity: LabelValueDefinitionSeverity::None,
            account: ExpectedBehaviors {
                profile_list: vec![Filter],
                content_list: vec![Filter],
                ..Default::default()
            },
            profile: ExpectedBehaviors::default(),
            post: ExpectedBehaviors {
                content_list: vec![Filter],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::Media,
            severity: LabelValueDefinitionSeverity::Alert,
            account: ExpectedBehaviors {
                profile_list: vec![Filter],
                content_list: vec![Filter],
                ..Default::default()
            },
            profile: ExpectedBehaviors::default(),
            post: ExpectedBehaviors {
                content_list: vec![Filter],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::Media,
            severity: LabelValueDefinitionSeverity::Inform,
            account: ExpectedBehaviors {
                profile_list: vec![Filter],
                content_list: vec![Filter],
                ..Default::default()
            },
            profile: ExpectedBehaviors::default(),
            post: ExpectedBehaviors {
                content_list: vec![Filter],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::Media,
            severity: LabelValueDefinitionSeverity::None,
            account: ExpectedBehaviors {
                profile_list: vec![Filter],
                content_list: vec![Filter],
                ..Default::default()
            },
            profile: ExpectedBehaviors::default(),
            post: ExpectedBehaviors {
                content_list: vec![Filter],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::None,
            severity: LabelValueDefinitionSeverity::Alert,
            account: ExpectedBehaviors {
                profile_list: vec![Filter],
                content_list: vec![Filter],
                ..Default::default()
            },
            profile: ExpectedBehaviors::default(),
            post: ExpectedBehaviors {
                content_list: vec![Filter],
                ..Default::default()
            },
        },
        Scenario {
            blurs: LabelValueDefinitionBlurs::None,
            severity: LabelValueDefinitionSeverity::Inform,
            account: ExpectedBehaviors {
                profile_list: vec![Filter],
                content_list: vec![Filter],
                ..Default::default()
            },
            profile: ExpectedBehaviors::default(),
            post: ExpectedBehaviors {
                content_list: vec![Filter],
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
            profile: ExpectedBehaviors::default(),
            post: ExpectedBehaviors {
                content_list: vec![Filter],
                ..Default::default()
            },
        },
    ];
    for scenario in scenarios {
        scenario.run();
    }
}
