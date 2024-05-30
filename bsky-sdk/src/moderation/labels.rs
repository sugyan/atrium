use super::types::*;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KnownLabelValue {
    ReservedHide,
    ReservedWarn,
    ReservedNoUnauthenticated,
    Porn,
    Sexual,
    Nudity,
    GraphicMedia,
}

impl FromStr for KnownLabelValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "!hide" => Ok(Self::ReservedHide),
            "!warn" => Ok(Self::ReservedWarn),
            "!no-unauthenticated" => Ok(Self::ReservedNoUnauthenticated),
            "porn" => Ok(Self::Porn),
            "sexual" => Ok(Self::Sexual),
            "nudity" => Ok(Self::Nudity),
            "graphic-media" => Ok(Self::GraphicMedia),
            _ => Err(()),
        }
    }
}

impl KnownLabelValue {
    pub fn definition(&self) -> InterpretedLabelValueDefinition {
        match self {
            Self::ReservedHide => InterpretedLabelValueDefinition {
                identifier: String::from("!hide"),
                default_setting: LabelPreference::Hide,
                flags: vec![
                    LabelValueDefinitionFlag::NoOverride,
                    LabelValueDefinitionFlag::NoSelf,
                ],
                behaviors: InterpretedLabelValueDefinitionBehaviors {
                    account: ModerationBehavior {
                        profile_list: Some(ProfileListBehavior::Blur),
                        profile_view: Some(ProfileViewBehavior::Blur),
                        avatar: Some(AvatarBehavior::Blur),
                        banner: Some(BannerBehavior::Blur),
                        display_name: Some(DisplayNameBehavior::Blur),
                        content_list: Some(ContentListBehavior::Blur),
                        content_view: Some(ContentViewBehavior::Blur),
                        ..Default::default()
                    },
                    profile: ModerationBehavior {
                        avatar: Some(AvatarBehavior::Blur),
                        banner: Some(BannerBehavior::Blur),
                        display_name: Some(DisplayNameBehavior::Blur),
                        ..Default::default()
                    },
                    content: ModerationBehavior {
                        content_list: Some(ContentListBehavior::Blur),
                        content_view: Some(ContentViewBehavior::Blur),
                        ..Default::default()
                    },
                },
            },
            Self::ReservedWarn => InterpretedLabelValueDefinition {
                identifier: String::from("!warn"),
                default_setting: LabelPreference::Warn,
                flags: vec![LabelValueDefinitionFlag::NoSelf],
                behaviors: InterpretedLabelValueDefinitionBehaviors {
                    account: ModerationBehavior {
                        profile_list: Some(ProfileListBehavior::Blur),
                        profile_view: Some(ProfileViewBehavior::Blur),
                        avatar: Some(AvatarBehavior::Blur),
                        banner: Some(BannerBehavior::Blur),
                        content_list: Some(ContentListBehavior::Blur),
                        content_view: Some(ContentViewBehavior::Blur),
                        ..Default::default()
                    },
                    profile: ModerationBehavior {
                        avatar: Some(AvatarBehavior::Blur),
                        banner: Some(BannerBehavior::Blur),
                        display_name: Some(DisplayNameBehavior::Blur),
                        ..Default::default()
                    },
                    content: ModerationBehavior {
                        content_list: Some(ContentListBehavior::Blur),
                        content_view: Some(ContentViewBehavior::Blur),
                        ..Default::default()
                    },
                },
            },
            Self::ReservedNoUnauthenticated => InterpretedLabelValueDefinition {
                identifier: String::from("!no-unauthenticated"),
                default_setting: LabelPreference::Hide,
                flags: vec![
                    LabelValueDefinitionFlag::NoOverride,
                    LabelValueDefinitionFlag::Unauthed,
                ],
                behaviors: InterpretedLabelValueDefinitionBehaviors {
                    account: ModerationBehavior {
                        profile_list: Some(ProfileListBehavior::Blur),
                        profile_view: Some(ProfileViewBehavior::Blur),
                        avatar: Some(AvatarBehavior::Blur),
                        banner: Some(BannerBehavior::Blur),
                        display_name: Some(DisplayNameBehavior::Blur),
                        content_list: Some(ContentListBehavior::Blur),
                        content_view: Some(ContentViewBehavior::Blur),
                        ..Default::default()
                    },
                    profile: ModerationBehavior {
                        avatar: Some(AvatarBehavior::Blur),
                        banner: Some(BannerBehavior::Blur),
                        display_name: Some(DisplayNameBehavior::Blur),
                        ..Default::default()
                    },
                    content: ModerationBehavior {
                        content_list: Some(ContentListBehavior::Blur),
                        content_view: Some(ContentViewBehavior::Blur),
                        ..Default::default()
                    },
                },
            },
            Self::Porn => InterpretedLabelValueDefinition {
                identifier: String::from("porn"),
                default_setting: LabelPreference::Hide,
                flags: vec![LabelValueDefinitionFlag::Adult],
                behaviors: InterpretedLabelValueDefinitionBehaviors {
                    account: ModerationBehavior {
                        avatar: Some(AvatarBehavior::Blur),
                        banner: Some(BannerBehavior::Blur),
                        ..Default::default()
                    },
                    profile: ModerationBehavior {
                        avatar: Some(AvatarBehavior::Blur),
                        banner: Some(BannerBehavior::Blur),
                        ..Default::default()
                    },
                    content: ModerationBehavior {
                        content_media: Some(ContentMediaBehavior::Blur),
                        ..Default::default()
                    },
                },
            },
            _ => todo!(),
        }
    }
}
