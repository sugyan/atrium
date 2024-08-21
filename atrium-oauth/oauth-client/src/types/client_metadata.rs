use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OAuthClientMetadata {
    pub client_id: String,
    pub redirect_uris: Vec<String>,
    pub token_endpoint_auth_method: Option<String>,
}
