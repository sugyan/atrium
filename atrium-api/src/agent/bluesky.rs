//! Bluesky specific constants.

/// DID of the bluesky labeler service.
pub const BSKY_LABELER_DID: &str = "did:plc:ar7c4by46qjdydhdevvrndac";
/// DID of the bluesky chat service.
pub const BSKY_CHAT_DID: &str = "did:web:api.bsky.chat";

/// Supported proxy targets, which includes the bluesky specific services.
pub enum AtprotoServiceType {
    AtprotoLabeler,
    BskyChat,
}

impl AsRef<str> for AtprotoServiceType {
    fn as_ref(&self) -> &str {
        match self {
            Self::AtprotoLabeler => "atproto_labeler",
            Self::BskyChat => "bsky_chat",
        }
    }
}
