mod atp_agent;
#[cfg(feature = "bluesky")]
pub mod bluesky;

pub use atp_agent::{AtpAgent, CredentialSession};

/// Supported proxy targets.
#[cfg(feature = "bluesky")]
pub type AtprotoServiceType = self::bluesky::AtprotoServiceType;

#[cfg(not(feature = "bluesky"))]
pub enum AtprotoServiceType {
    AtprotoLabeler,
}

#[cfg(not(feature = "bluesky"))]
impl AsRef<str> for AtprotoServiceType {
    fn as_ref(&self) -> &str {
        match self {
            Self::AtprotoLabeler => "atproto_labeler",
        }
    }
}
