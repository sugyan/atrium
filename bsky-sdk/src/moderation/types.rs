use super::decision::{DecisionContext, LabelTarget, Priority};
use atrium_api::agent::bluesky::BSKY_LABELER_DID;
use atrium_api::app::bsky::actor::defs::{ProfileView, ProfileViewBasic, ProfileViewDetailed};
use atrium_api::app::bsky::feed::defs::PostView;
use atrium_api::app::bsky::graph::defs::ListViewBasic;
use atrium_api::com::atproto::label::defs::Label;
use atrium_api::types::string::Did;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

// labels

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LabelValueDefinitionFlag {
    NoOverride,
    Adult,
    Unauthed,
    NoSelf,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LabelPreference {
    Ignore,
    Warn,
    Hide,
}

impl FromStr for LabelPreference {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ignore" => Ok(Self::Ignore),
            "warn" => Ok(Self::Warn),
            "hide" => Ok(Self::Hide),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InterpretedLabelValueDefinition {
    pub identifier: String,
    pub default_setting: LabelPreference,
    pub flags: Vec<LabelValueDefinitionFlag>,
    pub behaviors: InterpretedLabelValueDefinitionBehaviors,
    // TODO
}

#[derive(Debug, Default, Clone)]
pub(crate) struct InterpretedLabelValueDefinitionBehaviors {
    pub account: ModerationBehavior,
    pub profile: ModerationBehavior,
    pub content: ModerationBehavior,
}

impl InterpretedLabelValueDefinitionBehaviors {
    pub fn behavior_for(&self, target: LabelTarget) -> ModerationBehavior {
        match target {
            LabelTarget::Account => self.account.clone(),
            LabelTarget::Profile => self.profile.clone(),
            LabelTarget::Content => self.content.clone(),
        }
    }
}

// subjects

#[derive(Debug)]
pub enum SubjectProfile {
    ProfileViewBasic(ProfileViewBasic),
    ProfileView(ProfileView),
    ProfileViewDetailed(ProfileViewDetailed),
}

impl SubjectProfile {
    pub fn did(&self) -> &Did {
        match self {
            Self::ProfileViewBasic(p) => &p.did,
            Self::ProfileView(p) => &p.did,
            Self::ProfileViewDetailed(p) => &p.did,
        }
    }
    pub fn labels(&self) -> &Option<Vec<Label>> {
        match self {
            Self::ProfileViewBasic(p) => &p.labels,
            Self::ProfileView(p) => &p.labels,
            Self::ProfileViewDetailed(p) => &p.labels,
        }
    }
}

impl From<ProfileViewBasic> for SubjectProfile {
    fn from(p: ProfileViewBasic) -> Self {
        Self::ProfileViewBasic(p)
    }
}

impl From<ProfileView> for SubjectProfile {
    fn from(p: ProfileView) -> Self {
        Self::ProfileView(p)
    }
}

impl From<ProfileViewDetailed> for SubjectProfile {
    fn from(p: ProfileViewDetailed) -> Self {
        Self::ProfileViewDetailed(p)
    }
}

pub type SubjectPost = PostView;

// behaviors

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BehaviorValue {
    Blur,
    Alert,
    Inform,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ModerationBehavior {
    pub profile_list: Option<ProfileListBehavior>,
    pub profile_view: Option<ProfileViewBehavior>,
    pub avatar: Option<AvatarBehavior>,
    pub banner: Option<BannerBehavior>,
    pub display_name: Option<DisplayNameBehavior>,
    pub content_list: Option<ContentListBehavior>,
    pub content_view: Option<ContentViewBehavior>,
    pub content_media: Option<ContentMediaBehavior>,
}

impl ModerationBehavior {
    pub fn behavior_for(&self, context: DecisionContext) -> Option<BehaviorValue> {
        match context {
            DecisionContext::ProfileList => self.profile_list.clone().map(Into::into),
            DecisionContext::ProfileView => self.profile_view.clone().map(Into::into),
            DecisionContext::Avatar => self.avatar.clone().map(Into::into),
            DecisionContext::Banner => self.banner.clone().map(Into::into),
            DecisionContext::DisplayName => self.display_name.clone().map(Into::into),
            DecisionContext::ContentList => self.content_list.clone().map(Into::into),
            DecisionContext::ContentView => self.content_view.clone().map(Into::into),
            DecisionContext::ContentMedia => self.content_media.clone().map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProfileListBehavior {
    Blur,
    Alert,
    Inform,
}

impl From<ProfileListBehavior> for BehaviorValue {
    fn from(b: ProfileListBehavior) -> Self {
        match b {
            ProfileListBehavior::Blur => Self::Blur,
            ProfileListBehavior::Alert => Self::Alert,
            ProfileListBehavior::Inform => Self::Inform,
        }
    }
}

impl TryFrom<BehaviorValue> for ProfileListBehavior {
    type Error = ();

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            BehaviorValue::Alert => Ok(Self::Alert),
            BehaviorValue::Inform => Ok(Self::Inform),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProfileViewBehavior {
    Blur,
    Alert,
    Inform,
}

impl From<ProfileViewBehavior> for BehaviorValue {
    fn from(b: ProfileViewBehavior) -> Self {
        match b {
            ProfileViewBehavior::Blur => Self::Blur,
            ProfileViewBehavior::Alert => Self::Alert,
            ProfileViewBehavior::Inform => Self::Inform,
        }
    }
}

impl TryFrom<BehaviorValue> for ProfileViewBehavior {
    type Error = ();

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            BehaviorValue::Alert => Ok(Self::Alert),
            BehaviorValue::Inform => Ok(Self::Inform),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AvatarBehavior {
    Blur,
    Alert,
}

impl From<AvatarBehavior> for BehaviorValue {
    fn from(b: AvatarBehavior) -> Self {
        match b {
            AvatarBehavior::Blur => Self::Blur,
            AvatarBehavior::Alert => Self::Alert,
        }
    }
}

impl TryFrom<BehaviorValue> for AvatarBehavior {
    type Error = ();

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            BehaviorValue::Alert => Ok(Self::Alert),
            BehaviorValue::Inform => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BannerBehavior {
    Blur,
}

impl From<BannerBehavior> for BehaviorValue {
    fn from(b: BannerBehavior) -> Self {
        match b {
            BannerBehavior::Blur => Self::Blur,
        }
    }
}

impl TryFrom<BehaviorValue> for BannerBehavior {
    type Error = ();

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DisplayNameBehavior {
    Blur,
}

impl From<DisplayNameBehavior> for BehaviorValue {
    fn from(b: DisplayNameBehavior) -> Self {
        match b {
            DisplayNameBehavior::Blur => Self::Blur,
        }
    }
}

impl TryFrom<BehaviorValue> for DisplayNameBehavior {
    type Error = ();

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ContentListBehavior {
    Blur,
    Alert,
    Inform,
}

impl From<ContentListBehavior> for BehaviorValue {
    fn from(b: ContentListBehavior) -> Self {
        match b {
            ContentListBehavior::Blur => Self::Blur,
            ContentListBehavior::Alert => Self::Alert,
            ContentListBehavior::Inform => Self::Inform,
        }
    }
}

impl TryFrom<BehaviorValue> for ContentListBehavior {
    type Error = ();

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            BehaviorValue::Alert => Ok(Self::Alert),
            BehaviorValue::Inform => Ok(Self::Inform),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ContentViewBehavior {
    Blur,
    Alert,
    Inform,
}

impl From<ContentViewBehavior> for BehaviorValue {
    fn from(b: ContentViewBehavior) -> Self {
        match b {
            ContentViewBehavior::Blur => Self::Blur,
            ContentViewBehavior::Alert => Self::Alert,
            ContentViewBehavior::Inform => Self::Inform,
        }
    }
}

impl TryFrom<BehaviorValue> for ContentViewBehavior {
    type Error = ();

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            BehaviorValue::Alert => Ok(Self::Alert),
            BehaviorValue::Inform => Ok(Self::Inform),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ContentMediaBehavior {
    Blur,
}

impl From<ContentMediaBehavior> for BehaviorValue {
    fn from(b: ContentMediaBehavior) -> Self {
        match b {
            ContentMediaBehavior::Blur => Self::Blur,
        }
    }
}

impl TryFrom<BehaviorValue> for ContentMediaBehavior {
    type Error = ();

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ModerationCause {
    Blocking(
        //TODO
    ),
    BlockedBy(
        //TODO
    ),
    BlockOther(
        //TODO
    ),
    Label(Box<ModerationCauseLabel>),
    Muted(
        //TODO
    ),
    MuteWord(
        //TODO
    ),
    Hidden(
        //TODO
    ),
}

impl ModerationCause {
    pub fn downgrade(&mut self) {
        match self {
            Self::Label(label) => label.downgraded = Some(true),
            _ => todo!(),
        }
    }
    pub fn priority(&self) -> Priority {
        match self {
            Self::Label(label) => label.priority,
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ModerationCauseSource {
    User,
    List(ListViewBasic),
    Labeler(Did),
}

#[derive(Debug, Clone)]
pub(crate) struct ModerationCauseLabel {
    pub source: ModerationCauseSource,
    pub label: Label,
    pub label_def: InterpretedLabelValueDefinition,
    pub target: LabelTarget,
    pub setting: LabelPreference,
    pub behavior: ModerationBehavior,
    pub no_override: bool,
    pub priority: Priority,
    pub downgraded: Option<bool>,
}

// moderation preferences

#[derive(Debug, Serialize, Deserialize)]
pub struct ModerationPrefsLabeler {
    pub did: Did,
    pub labels: HashMap<String, LabelPreference>,
    #[serde(skip_serializing)]
    pub is_default_labeler: bool,
}

impl Default for ModerationPrefsLabeler {
    fn default() -> Self {
        Self {
            did: BSKY_LABELER_DID.parse().expect("invalid did"),
            labels: HashMap::default(),
            is_default_labeler: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModerationPrefs {
    pub adult_content_enabled: bool,
    pub labels: HashMap<String, LabelPreference>,
    pub labelers: Vec<ModerationPrefsLabeler>,
}

impl Default for ModerationPrefs {
    fn default() -> Self {
        Self {
            adult_content_enabled: false,
            labels: HashMap::from_iter([
                (String::from("porn"), LabelPreference::Hide),
                (String::from("sexual"), LabelPreference::Warn),
                (String::from("nudity"), LabelPreference::Ignore),
                (String::from("graphic-media"), LabelPreference::Warn),
            ]),
            labelers: Vec::default(),
        }
    }
}
