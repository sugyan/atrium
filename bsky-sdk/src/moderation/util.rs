use super::types::*;
use crate::Result;
use atrium_api::app::bsky::labeler::defs::LabelerViewDetailed;
use atrium_api::com::atproto::label::defs::LabelValueDefinition;
use atrium_api::types::string::Did;

pub(crate) fn interpret_label_value_definitions(
    labeler_view: &LabelerViewDetailed,
) -> Result<Vec<InterpretedLabelValueDefinition>> {
    let defined_by = Some(labeler_view.creator.did.clone());
    labeler_view
        .policies
        .label_value_definitions
        .as_ref()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|label_value_definition| {
            interpret_label_value_definition(label_value_definition, defined_by.clone())
        })
        .collect()
}

pub fn interpret_label_value_definition(
    def: &LabelValueDefinition,
    defined_by: Option<Did>,
) -> Result<InterpretedLabelValueDefinition> {
    let adult_only = def.adult_only.unwrap_or_default();
    let blurs = def.blurs.parse()?;
    let default_setting =
        if let Some(pref) = def.default_setting.as_ref().and_then(|s| s.parse().ok()) {
            pref
        } else {
            LabelPreference::Warn
        };
    let severity = def.severity.parse()?;
    let mut flags = vec![LabelValueDefinitionFlag::NoSelf];
    if adult_only {
        flags.push(LabelValueDefinitionFlag::Adult);
    }
    let aoi = match severity {
        LabelValueDefinitionSeverity::Alert => Some(BehaviorValue::Alert),
        LabelValueDefinitionSeverity::Inform => Some(BehaviorValue::Inform),
        LabelValueDefinitionSeverity::None => None,
    };
    let mut behaviors = InterpretedLabelValueDefinitionBehaviors::default();
    match blurs {
        LabelValueDefinitionBlurs::Content => {
            // target=account, blurs=content
            behaviors.account.profile_list = aoi.map(BehaviorValue::try_into).transpose()?;
            behaviors.account.profile_view = aoi.map(BehaviorValue::try_into).transpose()?;
            behaviors.account.content_list = Some(ContentListBehavior::Blur);
            behaviors.account.content_view = if adult_only {
                Some(ContentViewBehavior::Blur)
            } else {
                aoi.map(BehaviorValue::try_into).transpose()?
            };
            // target=profile, blurs=content
            behaviors.profile.profile_list = aoi.map(BehaviorValue::try_into).transpose()?;
            behaviors.profile.profile_view = aoi.map(BehaviorValue::try_into).transpose()?;
            // target=content, blurs=content
            behaviors.content.content_list = Some(ContentListBehavior::Blur);
            behaviors.content.content_view = if adult_only {
                Some(ContentViewBehavior::Blur)
            } else {
                aoi.map(BehaviorValue::try_into).transpose()?
            };
        }
        LabelValueDefinitionBlurs::Media => {
            // target=account, blurs=media
            behaviors.account.profile_list = aoi.map(BehaviorValue::try_into).transpose()?;
            behaviors.account.profile_view = aoi.map(BehaviorValue::try_into).transpose()?;
            behaviors.account.avatar = Some(AvatarBehavior::Blur);
            behaviors.account.banner = Some(BannerBehavior::Blur);
            // target=profile, blurs=media
            behaviors.profile.profile_list = aoi.map(BehaviorValue::try_into).transpose()?;
            behaviors.profile.profile_view = aoi.map(BehaviorValue::try_into).transpose()?;
            behaviors.profile.avatar = Some(AvatarBehavior::Blur);
            behaviors.profile.banner = Some(BannerBehavior::Blur);
            // target=content, blurs=media
            behaviors.content.content_media = Some(ContentMediaBehavior::Blur);
        }
        LabelValueDefinitionBlurs::None => {
            // target=account, blurs=none
            behaviors.account.profile_list = aoi.map(BehaviorValue::try_into).transpose()?;
            behaviors.account.profile_view = aoi.map(BehaviorValue::try_into).transpose()?;
            behaviors.account.content_list = aoi.map(BehaviorValue::try_into).transpose()?;
            behaviors.account.content_view = aoi.map(BehaviorValue::try_into).transpose()?;
            // target=profile, blurs=none
            behaviors.profile.profile_list = aoi.map(BehaviorValue::try_into).transpose()?;
            behaviors.profile.profile_view = aoi.map(BehaviorValue::try_into).transpose()?;
            // target=content, blurs=none
            behaviors.content.content_list = aoi.map(BehaviorValue::try_into).transpose()?;
            behaviors.content.content_view = aoi.map(BehaviorValue::try_into).transpose()?;
        }
    }
    Ok(InterpretedLabelValueDefinition {
        adult_only,
        blurs,
        default_setting,
        identifier: def.identifier.clone(),
        locales: def.locales.clone(),
        severity,
        defined_by,
        configurable: true,
        flags,
        behaviors,
    })
}
