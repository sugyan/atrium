//! Definitions for DID document types.
//! https://atproto.com/specs/did#did-documents

/// A DID document, containing information associated with the DID
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
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

/// The public signing key for the account
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMethod {
    pub id: String,
    pub r#type: String,
    pub controller: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_multibase: Option<String>,
}

/// The PDS service network location for the account
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,
    pub r#type: String,
    pub service_endpoint: String,
}

impl TryFrom<&str> for DidDocument {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value)
    }
}

impl DidDocument {
    pub fn get_did(&self) -> String {
        self.id.clone()
    }
    pub fn get_handle(&self) -> Option<String> {
        if let Some(aka) = &self.also_known_as {
            aka.iter()
                .find_map(|name| name.strip_prefix("at://"))
                .map(String::from)
        } else {
            None
        }
    }
    pub fn get_signing_key(&self) -> Option<(String, String)> {
        self.get_verification_material("#atproto")
    }
    pub fn get_pds_endpoint(&self) -> Option<String> {
        self.get_service_endpoint("#atproto_pds", "AtprotoPersonalDataServer")
    }
    fn get_verification_material(&self, id: &str) -> Option<(String, String)> {
        let did = self.get_did();
        if let Some(keys) = &self.verification_method {
            keys.iter().find_map(|key| {
                if key.id == id || key.id == format!("{did}{id}") {
                    key.public_key_multibase
                        .as_ref()
                        .map(|multibase| (key.r#type.clone(), multibase.clone()))
                } else {
                    None
                }
            })
        } else {
            None
        }
    }
    fn get_service_endpoint(&self, id: &str, r#type: &str) -> Option<String> {
        let did = self.get_did();
        if let Some(services) = &self.service {
            services
                .iter()
                .find(|service| {
                    (service.id == id || service.id == format!("{did}{id}"))
                        && service.r#type == r#type
                })
                .and_then(|service| Self::validate_url(&service.service_endpoint))
        } else {
            None
        }
    }
    fn validate_url(s: &str) -> Option<String> {
        s.parse::<http::Uri>()
            .ok()
            .and_then(|uri| match (uri.scheme(), uri.host()) {
                (Some(scheme), Some(_)) if (scheme == "https" || scheme == "http") => {
                    Some(s.to_string())
                }
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DID_DOC_JSON: &str = r##"{"@context":["https://www.w3.org/ns/did/v1","https://w3id.org/security/multikey/v1","https://w3id.org/security/suites/secp256k1-2019/v1"],"id":"did:plc:4ee6oesrsbtmuln4gqsqf6fp","alsoKnownAs":["at://sugyan.com"],"verificationMethod":[{"id":"did:plc:4ee6oesrsbtmuln4gqsqf6fp#atproto","type":"Multikey","controller":"did:plc:4ee6oesrsbtmuln4gqsqf6fp","publicKeyMultibase":"zQ3shnw8ChQwGUE6gMghuvn5g7Q9YVej1MUJENqMsLmxZwRSz"}],"service":[{"id":"#atproto_pds","type":"AtprotoPersonalDataServer","serviceEndpoint":"https://puffball.us-east.host.bsky.network"}]}"##;

    fn did_doc_example() -> DidDocument {
        DidDocument {
            context: Some(vec![
                String::from("https://www.w3.org/ns/did/v1"),
                String::from("https://w3id.org/security/multikey/v1"),
                String::from("https://w3id.org/security/suites/secp256k1-2019/v1"),
            ]),
            id: String::from("did:plc:4ee6oesrsbtmuln4gqsqf6fp"),
            also_known_as: Some(vec![String::from("at://sugyan.com")]),
            verification_method: Some(vec![VerificationMethod {
                id: String::from("did:plc:4ee6oesrsbtmuln4gqsqf6fp#atproto"),
                r#type: String::from("Multikey"),
                controller: String::from("did:plc:4ee6oesrsbtmuln4gqsqf6fp"),
                public_key_multibase: Some(String::from(
                    "zQ3shnw8ChQwGUE6gMghuvn5g7Q9YVej1MUJENqMsLmxZwRSz",
                )),
            }]),
            service: Some(vec![Service {
                id: String::from("#atproto_pds"),
                r#type: String::from("AtprotoPersonalDataServer"),
                service_endpoint: String::from("https://puffball.us-east.host.bsky.network"),
            }]),
        }
    }

    #[test]
    fn serialize_did_doc() {
        let result =
            serde_json::to_string(&did_doc_example()).expect("serialization should succeed");
        assert_eq!(result, DID_DOC_JSON);
    }

    #[test]
    fn deserialize_did_doc() {
        let result = serde_json::from_str::<DidDocument>(DID_DOC_JSON)
            .expect("deserialization should succeed");
        assert_eq!(result, did_doc_example());
    }
}
