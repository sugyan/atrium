use super::{assert_ui, label, profile_view_basic, FAKE_CID};
use super::{post_view, ModerationTestResultFlag};
use crate::moderation::decision::DecisionContext;
use crate::moderation::types::*;
use crate::moderation::Moderator;
use atrium_api::app::bsky::actor::defs::{ProfileViewBasic, ViewerState};
use atrium_api::app::bsky::graph::defs::{ListPurpose, ListViewBasic};
use atrium_api::types::string::Datetime;
use std::collections::HashMap;

fn list_view_basic(name: &str) -> ListViewBasic {
    ListViewBasic {
        avatar: None,
        cid: FAKE_CID.parse().expect("invalid cid"),
        indexed_at: Some(Datetime::now()),
        labels: None,
        name: name.into(),
        purpose: ListPurpose::from("app.bsky.graph.defs#modlist"),
        uri: String::from("at://did:plc:fake/app.bsky.graph.list/fake"),
        viewer: None,
    }
}

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

impl TestUser {
    fn viewer_state(&self) -> ViewerState {
        #[derive(Default)]
        struct Definition {
            blocking: bool,
            blocking_by_list: bool,
            blocked_by: bool,
            muted: bool,
            muted_by_list: bool,
        }
        let def = match self {
            Self::Bob => Definition {
                blocking: true,
                ..Default::default()
            },
            Self::Carla => Definition {
                blocked_by: true,
                ..Default::default()
            },
            Self::Dan => Definition {
                muted: true,
                ..Default::default()
            },
            Self::Elise => Definition {
                muted_by_list: true,
                ..Default::default()
            },
            Self::Fern => Definition {
                blocking: true,
                blocked_by: true,
                ..Default::default()
            },
            Self::Georgia => Definition {
                blocking_by_list: true,
                ..Default::default()
            },
            _ => Definition::default(),
        };
        ViewerState {
            blocked_by: if def.blocked_by { Some(true) } else { None },
            blocking: if def.blocking || def.blocking_by_list {
                Some(String::from(
                    "at://did:web:self.test/app.bsky.graph.block/fake",
                ))
            } else {
                None
            },
            blocking_by_list: if def.blocking_by_list {
                Some(list_view_basic("Fake list"))
            } else {
                None
            },
            followed_by: None,
            following: None,
            muted: if def.muted || def.muted_by_list {
                Some(true)
            } else {
                None
            },
            muted_by_list: if def.muted_by_list {
                Some(list_view_basic("Fake list"))
            } else {
                None
            },
        }
    }
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
struct BehaviorsTestScenario {
    cfg: TestConfig,
    subject: TestSubject,
    author: TestUser,
    labels: TestScenarioLabels,
    behaviors: TestExpectedBehaviors,
}

impl BehaviorsTestScenario {
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
        Moderator::new(
            match self.cfg {
                TestConfig::LoggedOut => None,
                _ => Some("did:web:self.test".parse().expect("invalid did")),
            },
            ModerationPrefs {
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
                ..Default::default()
            },
            HashMap::new(),
        )
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
        let mut ret = profile_view_basic(
            &format!("{}.test", self.author.as_ref()),
            None,
            Some(labels),
        );
        ret.viewer = Some(self.author.viewer_state());
        ret
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
fn moderation_behaviors() {
    use ModerationTestResultFlag::*;
    let scenarios = [
        (
            "Imperative label ('!hide') on account",
            BehaviorsTestScenario {
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
            BehaviorsTestScenario {
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
            BehaviorsTestScenario {
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
            BehaviorsTestScenario {
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
            BehaviorsTestScenario {
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
            BehaviorsTestScenario {
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
            BehaviorsTestScenario {
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
            BehaviorsTestScenario {
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
            BehaviorsTestScenario {
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
            BehaviorsTestScenario {
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
            BehaviorsTestScenario {
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
        (
            "Imperative label ('!no-unauthenticated') on profile when logged out",
            BehaviorsTestScenario {
                cfg: TestConfig::LoggedOut,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("!no-unauthenticated")],
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
            "Imperative label ('!no-unauthenticated') on post when logged out",
            BehaviorsTestScenario {
                cfg: TestConfig::LoggedOut,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    post: vec![String::from("!no-unauthenticated")],
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
            "Imperative label ('!no-unauthenticated') on author profile when logged out",
            BehaviorsTestScenario {
                cfg: TestConfig::LoggedOut,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("!no-unauthenticated")],
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
            "Imperative label ('!no-unauthenticated') on author account when logged out",
            BehaviorsTestScenario {
                cfg: TestConfig::LoggedOut,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("!no-unauthenticated")],
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
            "Imperative label ('!no-unauthenticated') on account when logged in",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("!no-unauthenticated")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors::default(),
            },
        ),
        (
            "Imperative label ('!no-unauthenticated') on profile when logged in",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("!no-unauthenticated")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors::default(),
            },
        ),
        (
            "Imperative label ('!no-unauthenticated') on post when logged in",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    post: vec![String::from("!no-unauthenticated")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors::default(),
            },
        ),
        (
            "Imperative label ('!no-unauthenticated') on author profile when logged in",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("!no-unauthenticated")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors::default(),
            },
        ),
        (
            "Imperative label ('!no-unauthenticated') on author account when logged in",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("!no-unauthenticated")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors::default(),
            },
        ),
        (
            "Blur-media label ('porn') on account (hide)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornHide,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Filter],
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    content_list: vec![Filter],
                    ..Default::default()
                },
            },
        ),
        (
            "Blur-media label ('porn') on profile (hide)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornHide,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Blur-media label ('porn') on post (hide)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornHide,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    post: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    content_list: vec![Filter],
                    content_media: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Blur-media label ('porn') on author profile (hide)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornHide,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Blur-media label ('porn') on author account (hide)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornHide,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Filter],
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    content_list: vec![Filter],
                    ..Default::default()
                },
            },
        ),
        (
            "Blur-media label ('porn') on account (warn)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornWarn,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Blur-media label ('porn') on profile (warn)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornWarn,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Blur-media label ('porn') on post (warn)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornWarn,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    post: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    content_media: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Blur-media label ('porn') on author profile (warn)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornWarn,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Blur-media label ('porn') on author account (warn)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornWarn,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Blur-media label ('porn') on account (ignore)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornIgnore,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors::default(),
            },
        ),
        (
            "Blur-media label ('porn') on profile (ignore)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornIgnore,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors::default(),
            },
        ),
        (
            "Blur-media label ('porn') on post (ignore)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornIgnore,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    post: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors::default(),
            },
        ),
        (
            "Blur-media label ('porn') on author profile (ignore)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornIgnore,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors::default(),
            },
        ),
        (
            "Blur-media label ('porn') on author account (ignore)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornIgnore,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors::default(),
            },
        ),
        (
            "Adult-only label on account when adult content is disabled",
            BehaviorsTestScenario {
                cfg: TestConfig::AdultDisabled,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Filter],
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    content_list: vec![Filter],
                    ..Default::default()
                },
            },
        ),
        (
            "Adult-only label on profile when adult content is disabled",
            BehaviorsTestScenario {
                cfg: TestConfig::AdultDisabled,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Adult-only label on post when adult content is disabled",
            BehaviorsTestScenario {
                cfg: TestConfig::AdultDisabled,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    post: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    content_list: vec![Filter],
                    content_media: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Adult-only label on author profile when adult content is disabled",
            BehaviorsTestScenario {
                cfg: TestConfig::AdultDisabled,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    profile: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Adult-only label on author account when adult content is disabled",
            BehaviorsTestScenario {
                cfg: TestConfig::AdultDisabled,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    content_list: vec![Filter],
                    ..Default::default()
                },
            },
        ),
        (
            "Self-profile: !hide on account",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::UserSelf,
                labels: TestScenarioLabels {
                    account: vec![String::from("!hide")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Blur],
                    profile_view: vec![Blur],
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    display_name: vec![Blur],
                    content_list: vec![Blur],
                    content_view: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Self-profile: !hide on profile",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::UserSelf,
                labels: TestScenarioLabels {
                    profile: vec![String::from("!hide")],
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
            "Self-post: Imperative label ('!hide') on post",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::UserSelf,
                labels: TestScenarioLabels {
                    post: vec![String::from("!hide")],
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
            "Self-post: Imperative label ('!hide') on author profile",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::UserSelf,
                labels: TestScenarioLabels {
                    profile: vec![String::from("!hide")],
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
            "Self-post: Imperative label ('!hide') on author account",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::UserSelf,
                labels: TestScenarioLabels {
                    account: vec![String::from("!hide")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    display_name: vec![Blur],
                    content_list: vec![Blur],
                    content_view: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Self-post: Imperative label ('!warn') on post",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::UserSelf,
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
            "Self-post: Imperative label ('!warn') on author profile",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::UserSelf,
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
            "Self-post: Imperative label ('!warn') on author account",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::UserSelf,
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
            "Mute/block: Blocking user",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Bob,
                labels: TestScenarioLabels::default(),
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Filter, Blur, NoOverride],
                    profile_view: vec![Alert],
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    content_list: vec![Filter, Blur, NoOverride],
                    content_view: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Post with blocked author",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Bob,
                labels: TestScenarioLabels::default(),
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    content_list: vec![Filter, Blur, NoOverride],
                    content_view: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Post with author blocking user",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Carla,
                labels: TestScenarioLabels::default(),
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    content_list: vec![Filter, Blur, NoOverride],
                    content_view: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Mute/block: Blocking-by-list user",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Georgia,
                labels: TestScenarioLabels::default(),
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Filter, Blur, NoOverride],
                    profile_view: vec![Alert],
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    content_list: vec![Filter, Blur, NoOverride],
                    content_view: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Mute/block: Blocked by user",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Carla,
                labels: TestScenarioLabels::default(),
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Filter, Blur, NoOverride],
                    profile_view: vec![Alert],
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    content_list: vec![Filter, Blur, NoOverride],
                    content_view: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Mute/block: Muted user",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Dan,
                labels: TestScenarioLabels::default(),
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Filter, Inform],
                    profile_view: vec![Alert],
                    content_list: vec![Filter, Blur],
                    content_view: vec![Inform],
                    ..Default::default()
                },
            },
        ),
        (
            "Mute/block: Muted-by-list user",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Elise,
                labels: TestScenarioLabels::default(),
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Filter, Inform],
                    profile_view: vec![Alert],
                    content_list: vec![Filter, Blur],
                    content_view: vec![Inform],
                    ..Default::default()
                },
            },
        ),
        (
            "Merging: blocking & blocked-by user",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Fern,
                labels: TestScenarioLabels::default(),
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Filter, Blur, NoOverride],
                    profile_view: vec![Alert],
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    content_list: vec![Filter, Blur, NoOverride],
                    content_view: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Post with muted author",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Dan,
                labels: TestScenarioLabels::default(),
                behaviors: TestExpectedBehaviors {
                    content_list: vec![Filter, Blur],
                    content_view: vec![Inform],
                    ..Default::default()
                },
            },
        ),
        (
            "Post with muted-by-list author",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Elise,
                labels: TestScenarioLabels::default(),
                behaviors: TestExpectedBehaviors {
                    content_list: vec![Filter, Blur],
                    content_view: vec![Inform],
                    ..Default::default()
                },
            },
        ),
        (
            "Merging: '!hide' label on account of blocked user",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Bob,
                labels: TestScenarioLabels {
                    account: vec![String::from("!hide")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Filter, Blur, NoOverride],
                    profile_view: vec![Blur, Alert, NoOverride],
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
            "Merging: '!hide' and 'porn' labels on account (hide)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornHide,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("!hide"), String::from("porn")],
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
            "Merging: '!warn' and 'porn' labels on account (hide)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornHide,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("!warn"), String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Filter, Blur],
                    profile_view: vec![Blur],
                    avatar: vec![Blur],
                    banner: vec![Blur],
                    content_list: vec![Filter, Blur],
                    content_view: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Merging: !hide on account, !warn on profile",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("!hide")],
                    profile: vec![String::from("!warn")],
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
            "Merging: !warn on account, !hide on profile",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Profile,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    account: vec![String::from("!warn")],
                    profile: vec![String::from("!hide")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    profile_list: vec![Blur],
                    profile_view: vec![Blur],
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    display_name: vec![Blur, NoOverride],
                    content_list: vec![Blur],
                    content_view: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Merging: post with blocking & blocked-by author",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Fern,
                labels: TestScenarioLabels::default(),
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    content_list: vec![Filter, Blur, NoOverride],
                    content_view: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Merging: '!hide' label on post by blocked user",
            BehaviorsTestScenario {
                cfg: TestConfig::None,
                subject: TestSubject::Post,
                author: TestUser::Bob,
                labels: TestScenarioLabels {
                    post: vec![String::from("!hide")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    avatar: vec![Blur, NoOverride],
                    banner: vec![Blur, NoOverride],
                    content_list: vec![Filter, Blur, NoOverride],
                    content_view: vec![Blur, NoOverride],
                    ..Default::default()
                },
            },
        ),
        (
            "Merging: '!hide' and 'porn' labels on post (hide)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornHide,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    post: vec![String::from("!warn"), String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    content_list: vec![Filter, Blur],
                    content_view: vec![Blur],
                    content_media: vec![Blur],
                    ..Default::default()
                },
            },
        ),
        (
            "Merging: '!warn' and 'porn' labels on post (hide)",
            BehaviorsTestScenario {
                cfg: TestConfig::PornHide,
                subject: TestSubject::Post,
                author: TestUser::Alice,
                labels: TestScenarioLabels {
                    post: vec![String::from("!warn"), String::from("porn")],
                    ..Default::default()
                },
                behaviors: TestExpectedBehaviors {
                    content_list: vec![Filter, Blur],
                    content_view: vec![Blur],
                    content_media: vec![Blur],
                    ..Default::default()
                },
            },
        ),
    ];
    for (_, scenario) in scenarios {
        scenario.run();
    }
}
