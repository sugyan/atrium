use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OAuthParResponse {
    pub request_uri: String,
    pub expires_in: Option<u32>,
}
