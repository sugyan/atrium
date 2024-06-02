use super::decision::{DecisionContext, Priority};
use atrium_api::agent::bluesky::BSKY_LABELER_DID;
use atrium_api::app::bsky::actor::defs::{
    MutedWord, ProfileView, ProfileViewBasic, ProfileViewDetailed, ViewerState,
};
use atrium_api::app::bsky::feed::defs::PostView;
use atrium_api::app::bsky::graph::defs::ListViewBasic;
use atrium_api::com::atproto::label::defs::{Label, LabelValueDefinitionStrings};
use atrium_api::types::string::Did;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

// errors

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid label preference")]
    LabelPreference,
    #[error("invalid label value definition blurs")]
    LabelValueDefinitionBlurs,
    #[error("invalid label value definition severity")]
    LabelValueDefinitionSeverity,
    #[error("invalid behavior value")]
    BehaviorValue,
}

// behaviors

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BehaviorValue {
    Blur,
    Alert,
    Inform,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationBehavior {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_list: Option<ProfileListBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_view: Option<ProfileViewBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<AvatarBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<BannerBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<DisplayNameBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_list: Option<ContentListBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_view: Option<ContentViewBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_media: Option<ContentMediaBehavior>,
}

impl ModerationBehavior {
    pub(crate) const BLOCK_BEHAVIOR: Self = Self {
        profile_list: Some(ProfileListBehavior::Blur),
        profile_view: Some(ProfileViewBehavior::Alert),
        avatar: Some(AvatarBehavior::Blur),
        banner: Some(BannerBehavior::Blur),
        display_name: None,
        content_list: Some(ContentListBehavior::Blur),
        content_view: Some(ContentViewBehavior::Blur),
        content_media: None,
    };
    pub(crate) const MUTE_BEHAVIOR: Self = Self {
        profile_list: Some(ProfileListBehavior::Inform),
        profile_view: Some(ProfileViewBehavior::Alert),
        avatar: None,
        banner: None,
        display_name: None,
        content_list: Some(ContentListBehavior::Blur),
        content_view: Some(ContentViewBehavior::Inform),
        content_media: None,
    };
    pub(crate) fn behavior_for(&self, context: DecisionContext) -> Option<BehaviorValue> {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProfileListBehavior {
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
    type Error = Error;

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            BehaviorValue::Alert => Ok(Self::Alert),
            BehaviorValue::Inform => Ok(Self::Inform),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProfileViewBehavior {
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
    type Error = Error;

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            BehaviorValue::Alert => Ok(Self::Alert),
            BehaviorValue::Inform => Ok(Self::Inform),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AvatarBehavior {
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
    type Error = Error;

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            BehaviorValue::Alert => Ok(Self::Alert),
            BehaviorValue::Inform => Err(Error::BehaviorValue),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BannerBehavior {
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
    type Error = Error;

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            _ => Err(Error::BehaviorValue),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayNameBehavior {
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
    type Error = Error;

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            _ => Err(Error::BehaviorValue),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentListBehavior {
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
    type Error = Error;

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            BehaviorValue::Alert => Ok(Self::Alert),
            BehaviorValue::Inform => Ok(Self::Inform),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentViewBehavior {
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
    type Error = Error;

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            BehaviorValue::Alert => Ok(Self::Alert),
            BehaviorValue::Inform => Ok(Self::Inform),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentMediaBehavior {
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
    type Error = Error;

    fn try_from(b: BehaviorValue) -> Result<Self, Self::Error> {
        match b {
            BehaviorValue::Blur => Ok(Self::Blur),
            _ => Err(Error::BehaviorValue),
        }
    }
}

// labels

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelTarget {
    Account,
    Profile,
    Content,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LabelPreference {
    Ignore,
    Warn,
    Hide,
}

impl FromStr for LabelPreference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ignore" => Ok(Self::Ignore),
            "warn" => Ok(Self::Warn),
            "hide" => Ok(Self::Hide),
            _ => Err(Error::LabelPreference),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LabelValueDefinitionFlag {
    NoOverride,
    Adult,
    Unauthed,
    NoSelf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LabelValueDefinitionBlurs {
    Content,
    Media,
    None,
}

impl FromStr for LabelValueDefinitionBlurs {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "content" => Ok(Self::Content),
            "media" => Ok(Self::Media),
            "none" => Ok(Self::None),
            _ => Err(Error::LabelValueDefinitionBlurs),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LabelValueDefinitionSeverity {
    Inform,
    Alert,
    None,
}

impl FromStr for LabelValueDefinitionSeverity {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "inform" => Ok(Self::Inform),
            "alert" => Ok(Self::Alert),
            "none" => Ok(Self::None),
            _ => Err(Error::LabelValueDefinitionSeverity),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterpretedLabelValueDefinition {
    // from com.atproto.label/defs#labelValueDefinition, with type narrowing
    pub adult_only: bool,
    pub blurs: LabelValueDefinitionBlurs,
    pub default_setting: LabelPreference,
    pub identifier: String,
    pub locales: Vec<LabelValueDefinitionStrings>,
    pub severity: LabelValueDefinitionSeverity,
    // others
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_by: Option<Did>,
    pub configurable: bool,
    pub flags: Vec<LabelValueDefinitionFlag>,
    pub behaviors: InterpretedLabelValueDefinitionBehaviors,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InterpretedLabelValueDefinitionBehaviors {
    pub account: ModerationBehavior,
    pub profile: ModerationBehavior,
    pub content: ModerationBehavior,
}

impl InterpretedLabelValueDefinitionBehaviors {
    pub(crate) fn behavior_for(&self, target: LabelTarget) -> ModerationBehavior {
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
    pub(crate) fn did(&self) -> &Did {
        match self {
            Self::ProfileViewBasic(p) => &p.did,
            Self::ProfileView(p) => &p.did,
            Self::ProfileViewDetailed(p) => &p.did,
        }
    }
    pub(crate) fn labels(&self) -> &Option<Vec<Label>> {
        match self {
            Self::ProfileViewBasic(p) => &p.labels,
            Self::ProfileView(p) => &p.labels,
            Self::ProfileViewDetailed(p) => &p.labels,
        }
    }
    pub(crate) fn viewer(&self) -> &Option<ViewerState> {
        match self {
            Self::ProfileViewBasic(p) => &p.viewer,
            Self::ProfileView(p) => &p.viewer,
            Self::ProfileViewDetailed(p) => &p.viewer,
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

#[derive(Debug, Clone)]
pub(crate) enum ModerationCause {
    Blocking(Box<ModerationCauseOther>),
    BlockedBy(Box<ModerationCauseOther>),
    BlockOther(Box<ModerationCauseOther>),
    Label(Box<ModerationCauseLabel>),
    Muted(Box<ModerationCauseOther>),
    MuteWord(Box<ModerationCauseOther>),
    Hidden(Box<ModerationCauseOther>),
}

impl ModerationCause {
    pub fn priority(&self) -> Priority {
        match self {
            Self::Blocking(_) => Priority::Priority3,
            Self::BlockedBy(_) => Priority::Priority4,
            Self::Label(label) => label.priority,
            Self::Muted(_) => Priority::Priority6,
            _ => todo!(),
        }
    }
    pub fn downgrade(&mut self) {
        match self {
            Self::Blocking(blocking) => blocking.downgraded = true,
            Self::BlockedBy(blocked_by) => blocked_by.downgraded = true,
            Self::Label(label) => label.downgraded = true,
            Self::Muted(muted) => muted.downgraded = true,
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ModerationCauseSource {
    User,
    List(Box<ListViewBasic>),
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
    pub downgraded: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ModerationCauseOther {
    pub source: ModerationCauseSource,
    pub downgraded: bool,
}

// moderation preferences

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationPrefs {
    pub adult_content_enabled: bool,
    pub labels: HashMap<String, LabelPreference>,
    pub labelers: Vec<ModerationPrefsLabeler>,
    pub muted_words: Vec<MutedWord>,
    pub hidden_posts: Vec<String>,
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
            muted_words: Vec::default(),
            hidden_posts: Vec::default(),
        }
    }
}
