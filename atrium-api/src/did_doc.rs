#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DidDocument {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub also_known_as: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_method: Option<Vec<VerificationMethod>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Vec<Service>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMethod {
    pub id: String,
    pub r#type: String,
    pub controller: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_multibase: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,
    pub r#type: String,
    // TODO: enum?
    pub service_endpoint: String,
}
