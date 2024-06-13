//! UI representation of moderation.
use super::types::ModerationCause;

/// UI representation of moderation decision results.
pub struct ModerationUi {
    /// Should the UI disable opening the cover?
    pub no_override: bool,
    /// Reasons to filter the content
    pub filters: Vec<ModerationCause>,
    /// Reasons to blur the content
    pub blurs: Vec<ModerationCause>,
    /// Reasons to alert the content
    pub alerts: Vec<ModerationCause>,
    /// Reasons to inform the content
    pub informs: Vec<ModerationCause>,
}

impl ModerationUi {
    /// Should the content be removed from the interface?
    pub fn filter(&self) -> bool {
        !self.filters.is_empty()
    }
    /// Should the content be put behind a cover?
    pub fn blur(&self) -> bool {
        !self.blurs.is_empty()
    }
    /// Should an alert be put on the content? (negative)
    pub fn alert(&self) -> bool {
        !self.alerts.is_empty()
    }
    /// Should an informational notice be put on the content? (neutral)
    pub fn inform(&self) -> bool {
        !self.informs.is_empty()
    }
}
