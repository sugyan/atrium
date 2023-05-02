#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "$type", rename_all = "lowercase")]
pub enum BlobRef {
    Blob(Blob),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Blob {
    r#ref: CID,
    mime_type: String,
    size: usize, // TODO
}

// TODO
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CID {
    #[serde(rename = "$link")]
    link: String,
}
