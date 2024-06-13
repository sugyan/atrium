use thiserror::Error;

/// Error type for this module.
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

/// Type alias to use this module's [`Error`](enum@self::Error) type in a [`Result`](core::result::Result).
pub type Result<T> = std::result::Result<T, Error>;
