use super::types::*;
use super::{labels::KnownLabelValue, ui::ModerationUi, Moderator};
use atrium_api::app::bsky::graph::defs::ListViewBasic;
use atrium_api::com::atproto::label::defs::Label;
use atrium_api::types::string::Did;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionContext {
    ProfileList,
    ProfileView,
    Avatar,
    Banner,
    DisplayName,
    ContentList,
    ContentView,
    ContentMedia,
}

impl DecisionContext {
    pub const ALL: [DecisionContext; 8] = [
        DecisionContext::ProfileList,
        DecisionContext::ProfileView,
        DecisionContext::Avatar,
        DecisionContext::Banner,
        DecisionContext::DisplayName,
        DecisionContext::ContentList,
        DecisionContext::ContentView,
        DecisionContext::ContentMedia,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LabelTarget {
    Account,
    Profile,
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModerationBehaviorSeverity {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Priority {
    Priority1,
    Priority2,
    Priority3,
    Priority5,
    Priority7,
    Priority8,
}

#[derive(Debug)]
pub struct ModerationDecision {
    did: Option<Did>,
    is_me: bool,
    causes: Vec<ModerationCause>,
}

impl ModerationDecision {
    pub fn ui(&self, context: DecisionContext) -> ModerationUi {
        let mut ui = ModerationUi {
            no_override: false,
            filters: Vec::new(),
            blurs: Vec::new(),
            alerts: Vec::new(),
            informs: Vec::new(),
        };
        for cause in &self.causes {
            match cause {
                ModerationCause::Blocking(_)
                | ModerationCause::BlockedBy()
                | ModerationCause::BlockOther() => {
                    if self.is_me {
                        continue;
                    }
                    if matches!(
                        context,
                        DecisionContext::ProfileList | DecisionContext::ContentList
                    ) {
                        ui.filters.push(cause.clone())
                    }
                    if !cause.downgraded() {
                        match ModerationBehavior::BLOCK_BEHAVIOR.behavior_for(context) {
                            Some(BehaviorValue::Blur) => {
                                ui.no_override = true;
                                ui.blurs.push(cause.clone());
                            }
                            Some(BehaviorValue::Alert) => {
                                ui.alerts.push(cause.clone());
                            }
                            Some(BehaviorValue::Inform) => {
                                ui.informs.push(cause.clone());
                            }
                            _ => {}
                        }
                    }
                }
                ModerationCause::Label(label) => {
                    if matches!(
                        (context, label.target),
                        (DecisionContext::ProfileList, LabelTarget::Account)
                            | (
                                DecisionContext::ContentList,
                                LabelTarget::Account | LabelTarget::Content,
                            ),
                    ) && (label.setting == LabelPreference::Hide && !self.is_me)
                    {
                        ui.filters.push(cause.clone())
                    }
                    if !cause.downgraded() {
                        match label.behavior.behavior_for(context) {
                            Some(BehaviorValue::Blur) => {
                                ui.blurs.push(cause.clone());
                                if label.no_override && !self.is_me {
                                    ui.no_override = true;
                                }
                            }
                            Some(BehaviorValue::Alert) => {
                                ui.alerts.push(cause.clone());
                            }
                            Some(BehaviorValue::Inform) => {
                                ui.informs.push(cause.clone());
                            }
                            _ => {}
                        }
                    }
                }
                ModerationCause::Muted() => {
                    todo!();
                }
                ModerationCause::MuteWord() => {
                    todo!();
                }
                ModerationCause::Hidden() => {
                    todo!();
                }
            }
        }
        ui.filters.sort_by_cached_key(|c| c.priority());
        ui.blurs.sort_by_cached_key(|c| c.priority());
        ui
    }
    pub(crate) fn new() -> Self {
        Self {
            did: None,
            is_me: false,
            causes: Vec::new(),
        }
    }
    pub(crate) fn merge(decisions: &[Self]) -> Self {
        assert!(!decisions.is_empty());
        assert!(decisions
            .windows(2)
            .all(|w| w[0].did == w[1].did && w[0].is_me == w[1].is_me));
        Self {
            did: decisions[0].did.clone(),
            is_me: decisions[0].is_me,
            causes: decisions
                .iter()
                .flat_map(|d| d.causes.iter().cloned())
                .collect(),
        }
    }
    pub(crate) fn set_did(&mut self, did: Did) {
        self.did = Some(did);
    }
    pub(crate) fn set_is_me(&mut self, is_me: bool) {
        self.is_me = is_me;
    }
    pub(crate) fn add_label(&mut self, target: LabelTarget, label: &Label, moderator: &Moderator) {
        let Some(label_def) = Self::lookup_label_def(label, moderator) else {
            return;
        };
        let is_self = Some(&label.src) == self.did.as_ref();
        let labeler = if is_self {
            None
        } else {
            moderator.prefs.labelers.iter().find(|l| l.did == label.src)
        };
        if !is_self && labeler.is_none() {
            return; // skip labelers not configured by the user
        }
        if is_self && label_def.flags.contains(&LabelValueDefinitionFlag::NoSelf) {
            return; // skip self-labels that arent supported
        }

        // establish the label preference for interpretation
        let mut label_pref = label_def.default_setting;
        if label_def.flags.contains(&LabelValueDefinitionFlag::Adult)
            && !moderator.prefs.adult_content_enabled
        {
            label_pref = LabelPreference::Hide;
        } else if let Some(pref) = labeler.and_then(|l| l.labels.get(&label_def.identifier)) {
            label_pref = *pref;
        } else if let Some(pref) = moderator.prefs.labels.get(&label_def.identifier) {
            label_pref = *pref;
        }

        // ignore labels the user has asked to ignore
        if label_pref == LabelPreference::Ignore {
            return;
        }

        // ignore 'unauthed' labels when the user is authed
        if label_def
            .flags
            .contains(&LabelValueDefinitionFlag::Unauthed)
            && moderator.user_did.is_some()
        {
            return;
        }

        let behavior = label_def.behaviors.behavior_for(target);
        // establish the priority of the label
        let severity = Self::measure_moderation_behavior_severity(&behavior);
        let priority = if label_def
            .flags
            .contains(&LabelValueDefinitionFlag::NoOverride)
            || (label_def.flags.contains(&LabelValueDefinitionFlag::Adult)
                && !moderator.prefs.adult_content_enabled)
        {
            Priority::Priority1
        } else if label_pref == LabelPreference::Hide {
            Priority::Priority2
        } else if severity == ModerationBehaviorSeverity::High {
            // blurring profile view or content view
            Priority::Priority5
        } else if severity == ModerationBehaviorSeverity::Medium {
            // blurring content list or content media
            Priority::Priority7
        } else {
            // blurring avatar, adding alerts
            Priority::Priority8
        };

        let no_override = label_def
            .flags
            .contains(&LabelValueDefinitionFlag::NoOverride)
            || (label_def.flags.contains(&LabelValueDefinitionFlag::Adult)
                && !moderator.prefs.adult_content_enabled);

        self.causes
            .push(ModerationCause::Label(Box::new(ModerationCauseLabel {
                source: if is_self || labeler.is_none() {
                    ModerationCauseSource::User
                } else {
                    ModerationCauseSource::Labeler(label.src.clone())
                },
                label: label.clone(),
                label_def,
                target,
                setting: label_pref,
                behavior,
                no_override,
                priority,
                downgraded: None,
            })));
    }
    pub(crate) fn add_blocking_by_list(&mut self, list_view: &ListViewBasic) {
        todo!()
    }
    pub(crate) fn add_blocking(&mut self, blocking: &str) {
        self.causes.push(ModerationCause::Blocking(Box::new(
            ModerationCauseBlocking {
                source: ModerationCauseSource::User,
                priority: Priority::Priority3,
                downgraded: None,
            },
        )))
    }
    pub(crate) fn downgrade(&mut self) {
        for cause in self.causes.iter_mut() {
            cause.downgrade()
        }
    }
    fn lookup_label_def(
        label: &Label,
        moderator: &Moderator,
    ) -> Option<InterpretedLabelValueDefinition> {
        if label
            .val
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '-')
        {
            if let Some(def) = moderator
                .label_defs
                .as_ref()
                .and_then(|label_defs| label_defs.get(label.src.as_ref()))
                .and_then(|defs| defs.iter().find(|def| def.identifier == label.val))
            {
                return Some(def.clone());
            }
        }
        label
            .val
            .parse::<KnownLabelValue>()
            .ok()
            .map(|known_value| known_value.definition())
    }
    fn measure_moderation_behavior_severity(
        behavior: &ModerationBehavior,
    ) -> ModerationBehaviorSeverity {
        if behavior.profile_view == Some(ProfileViewBehavior::Blur)
            || behavior.content_view == Some(ContentViewBehavior::Blur)
        {
            return ModerationBehaviorSeverity::High;
        }
        if behavior.content_list == Some(ContentListBehavior::Blur)
            || behavior.content_media == Some(ContentMediaBehavior::Blur)
        {
            return ModerationBehaviorSeverity::Medium;
        }
        ModerationBehaviorSeverity::Low
    }
}
