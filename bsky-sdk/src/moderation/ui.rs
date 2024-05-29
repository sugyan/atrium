use super::types::ModerationCause;

pub struct ModerationUi {
    pub no_override: bool,
    pub(crate) filters: Vec<ModerationCause>,
    pub(crate) blurs: Vec<ModerationCause>,
    pub(crate) alerts: Vec<ModerationCause>,
    pub(crate) informs: Vec<ModerationCause>,
}

impl ModerationUi {
    pub fn filter(&self) -> bool {
        !self.filters.is_empty()
    }
    pub fn blur(&self) -> bool {
        !self.blurs.is_empty()
    }
    pub fn alert(&self) -> bool {
        !self.alerts.is_empty()
    }
    pub fn inform(&self) -> bool {
        !self.informs.is_empty()
    }
}
