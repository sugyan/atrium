// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `com.atproto.repo.deleteRecord` namespace."]
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    #[doc = "The NSID of the record collection."]
    pub collection: String,
    #[doc = "The handle or DID of the repo."]
    pub repo: String,
    #[doc = "The key of the record."]
    pub rkey: String,
    #[doc = "Compare and swap with the previous commit by cid."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_commit: Option<String>,
    #[doc = "Compare and swap with the previous record by cid."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_record: Option<String>,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {
    InvalidSwap(Option<String>),
}
