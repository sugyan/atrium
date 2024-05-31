use super::types::{
    InterpretedLabelValueDefinition, InterpretedLabelValueDefinitionBehaviors, LabelPreference,
};
use super::LabelValueDefinitionFlag;
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
    let severity = def.severity.parse()?;
    let default_setting =
        if let Some(pref) = def.default_setting.as_ref().and_then(|s| s.parse().ok()) {
            pref
        } else {
            LabelPreference::Warn
        };
    let mut flags = vec![LabelValueDefinitionFlag::NoSelf];
    if adult_only {
        flags.push(LabelValueDefinitionFlag::Adult);
    }
    let mut behaviors = InterpretedLabelValueDefinitionBehaviors::default();
    // TODO
    Ok(InterpretedLabelValueDefinition {
        adult_only,
        blurs: def.blurs.parse()?,
        default_setting,
        identifier: def.identifier.clone(),
        locales: def.locales.clone(),
        severity,
        defined_by,
        flags,
        behaviors,
    })
}
