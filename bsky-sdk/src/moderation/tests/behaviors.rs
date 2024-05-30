use atrium_api::app::bsky::actor::defs::ProfileViewBasic;

use super::{assert_ui, label, profile_view_basic};
use super::{post_view, ModerationTestResultFlag};
use crate::moderation::decision::DecisionContext;
use crate::moderation::types::*;
use crate::moderation::Moderator;
use std::collections::HashMap;
use std::ops::Sub;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestConfig {
    None,
    AdultDisabled,
    PornHide,
    PornWarn,
    PornIgnore,
    LoggedOut,
}

impl TestConfig {
    fn labels(&self) -> HashMap<String, LabelPreference> {
        match self {
            Self::PornHide => HashMap::from_iter([(String::from("porn"), LabelPreference::Hide)]),
            Self::PornWarn => HashMap::from_iter([(String::from("porn"), LabelPreference::Warn)]),
            Self::PornIgnore => {
                HashMap::from_iter([(String::from("porn"), LabelPreference::Ignore)])
            }
            _ => HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestSubject {
    Profile,
    Post,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestUser {
    UserSelf,
    Alice,
    Bob,
    Carla,
    Dan,
    Elise,
    Fern,
    Georgia,
}

impl AsRef<str> for TestUser {
    fn as_ref(&self) -> &str {
        match self {
            Self::UserSelf => "self",
            Self::Alice => "alice",
            Self::Bob => "bob",
            Self::Carla => "carla",
            Self::Dan => "dan",
            Self::Elise => "elise",
            Self::Fern => "fern",
            Self::Georgia => "georgia",
        }
    }
}

#[derive(Debug, Default)]
struct TestScenarioLabels {
    post: Vec<String>,
    profile: Vec<String>,
    account: Vec<String>,
    quoted_post: Vec<String>,
    quoted_account: Vec<String>,
}

#[derive(Debug, Default)]
struct TestExpectedBehaviors {
    profile_list: Vec<ModerationTestResultFlag>,
    profile_view: Vec<ModerationTestResultFlag>,
    avatar: Vec<ModerationTestResultFlag>,
    banner: Vec<ModerationTestResultFlag>,
    display_name: Vec<ModerationTestResultFlag>,
    content_list: Vec<ModerationTestResultFlag>,
    content_view: Vec<ModerationTestResultFlag>,
    content_media: Vec<ModerationTestResultFlag>,
}

#[derive(Debug)]
struct ModerationTestScenario {
    cfg: TestConfig,
    subject: TestSubject,
    author: TestUser,
    labels: TestScenarioLabels,
    behaviors: TestExpectedBehaviors,
}

impl ModerationTestScenario {
    fn run(&self) {
        let moderator = self.moderator();
        let result = match self.subject {
            TestSubject::Profile => moderator.moderate_profile(&self.profile().into()),
            TestSubject::Post => moderator.moderate_post(&self.post()),
        };
        if self.subject == TestSubject::Profile {
            assert_ui(
                &result,
                &self.behaviors.profile_list,
                DecisionContext::ProfileList,
            );
            assert_ui(
                &result,
                &self.behaviors.profile_view,
                DecisionContext::ProfileView,
            );
        }
        assert_ui(&result, &self.behaviors.avatar, DecisionContext::Avatar);
        assert_ui(&result, &self.behaviors.banner, DecisionContext::Banner);
        assert_ui(
            &result,
            &self.behaviors.display_name,
            DecisionContext::DisplayName,
        );
        assert_ui(
            &result,
            &self.behaviors.content_list,
            DecisionContext::ContentList,
        );
        assert_ui(
            &result,
            &self.behaviors.content_view,
            DecisionContext::ContentView,
        );
        assert_ui(
            &result,
            &self.behaviors.content_media,
            DecisionContext::ContentMedia,
        );
    }
    fn moderator(&self) -> Moderator {
        Moderator {
            user_did: match self.cfg {
                TestConfig::LoggedOut => None,
                _ => Some("did:web:self.test".parse().expect("invalid did")),
            },
            prefs: ModerationPrefs {
                adult_content_enabled: matches!(
                    self.cfg,
                    TestConfig::PornHide | TestConfig::PornWarn | TestConfig::PornIgnore
                ),
                labels: self.cfg.labels(),
                labelers: vec![ModerationPrefsLabeler {
                    did: "did:plc:fake-labeler".parse().expect("invalid did"),
                    labels: HashMap::new(),
                    is_default_labeler: false,
                }],
            },
            label_defs: None,
        }
    }
    fn profile(&self) -> ProfileViewBasic {
        let mut labels = Vec::new();
        for val in &self.labels.account {
            labels.push(label(
                "did:plc:fake-labeler",
                &format!("did:web:{}", self.author.as_ref()),
                val,
            ))
        }
        for val in &self.labels.profile {
            labels.push(label(
                "did:plc:fake-labeler",
                &format!(
                    "at://did:web:{}/app.bsky.actor.profile/self",
                    self.author.as_ref()
                ),
                val,
            ))
        }
        profile_view_basic(
            &format!("{}.test", self.author.as_ref()),
            None,
            Some(labels),
        )
    }
    fn post(&self) -> SubjectPost {
        let author = self.profile();
        post_view(
            &author,
            "Post text",
            Some(
                self.labels
                    .post
                    .iter()
                    .map(|val| {
                        label(
                            "did:plc:fake-labeler",
                            &format!("at://{}/app.bsky.feed.post/fake", author.did.as_ref()),
                            val,
                        )
                    })
                    .collect(),
            ),
        )
    }
}

#[test]
fn post_moderation_behaviors() {
    use ModerationTestResultFlag::*;
    let scenarios = [
        (
            "Imperative label ('!hide') on account",
            ModerationTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("!hide")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Filter, Blur, NoOverride],
                    profile_view: vec![Blur, NoOverride],
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    display_name: vec![Blur, NoOverride],
                    content_list: vec![Filter, Blur, NoOverride],
                    content_view: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Imperative label ('!hide') on profile",
            ModerationTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("!hide")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    display_name: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Imperative label ('!hide') on post",
            ModerationTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    post: vec![String::from("!hide")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    content_list: vec![Filter, Blur, NoOverride],
                    content_view: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Imperative label ('!hide') on author profile",
            ModerationTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("!hide")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    display_name: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Imperative label ('!hide') on author account",
            ModerationTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("!hide")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    display_name: vec![Blur, NoOverride],
                    content_list: vec![Filter, Blur, NoOverride],
                    content_view: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Imperative label ('!warn') on account",
            ModerationTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("!warn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Blur],
                    profile_view: vec![Blur],
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    content_list: vec![Blur],
                    content_view: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Imperative label ('!warn') on profile",
            ModerationTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("!warn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    display_name: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Imperative label ('!warn') on post",
            ModerationTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    post: vec![String::from("!warn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    content_list: vec![Blur],
                    content_view: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Imperative label ('!warn') on author profile",
            ModerationTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("!warn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    display_name: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Imperative label ('!warn') on author account",
            ModerationTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("!warn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    content_list: vec![Blur],
                    content_view: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Imperative label ('!no-unauthenticated') on account when logged out",
            ModerationTestScenario {
                cfg: TestConfig::LoggedOut,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("!no-unauthenticated")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Filter, Blur, NoOverride],
                    profile_view: vec![Blur, NoOverride],
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    display_name: vec![Blur, NoOverride],
                    content_list: vec![Filter, Blur, NoOverride],
                    content_view: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
    ];
    for (_, scenario) in scenarios {
        scenario.run();
    }
}
