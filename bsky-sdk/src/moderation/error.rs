use thiserror::Error;

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
    #[error("unknown label value")]
    KnownLabelValue,
}

pub type Result<T> = std::result::Result<T, Error>;
