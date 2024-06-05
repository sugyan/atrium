use super::types::ModerationCause;

pub struct ModerationUi {
    pub no_override: bool,
    pub filters: Vec<ModerationCause>,
    pub blurs: Vec<ModerationCause>,
    pub alerts: Vec<ModerationCause>,
    pub informs: Vec<ModerationCause>,
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
