//! Definitions for DID document types.
use http::{uri::Scheme, Uri};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DidDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@context")]
    pub context: Option<Vec<String>>,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub also_known_as: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_method: Option<Vec<VerificationMethod>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Vec<Service>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMethod {
    pub id: String,
    pub r#type: String,
    pub controller: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_multibase: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,
    pub r#type: String,
    pub service_endpoint: String,
}

impl DidDocument {
    pub fn get_pds_endpoint(&self) -> Option<String> {
        self.get_service_endpoint("#atproto_pds", "AtprotoPersonalDataServer")
    }
    pub fn get_feed_gen_endpoint(&self) -> Option<String> {
        self.get_service_endpoint("#bsky_fg", "BskyFeedGenerator")
    }
    pub fn get_notif_endpoint(&self) -> Option<String> {
        self.get_service_endpoint("#bsky_notif", "BskyNotificationService")
    }
    fn get_service_endpoint(&self, id: &str, r#type: &str) -> Option<String> {
        let full_id = self.id.to_string() + id;
        if let Some(services) = &self.service {
            let service_endpoint = services
                .iter()
                .find(|service| {
                    (service.id == id || service.id == full_id) && service.r#type == r#type
                })
                .map(|service| service.service_endpoint.clone())?;
            return Some(service_endpoint).filter(|s| Self::validate_url(s));
        }
        None
    }
    fn validate_url(url: &str) -> bool {
        url.parse::<Uri>()
            .map(|uri| match uri.scheme() {
                Some(scheme) if (scheme == &Scheme::HTTP || scheme == &Scheme::HTTPS) => {
                    uri.host().is_some()
                }
                _ => false,
            })
            .unwrap_or_default()
    }
    pub fn get_signing_key(&self) -> Option<&VerificationMethod> {
        self.verification_method.as_ref().and_then(|methods| {
            methods.iter().find(|method| {
                method.id == "#atproto" || method.id == format!("{}#atproto", self.id)
            })
        })
    }
}
