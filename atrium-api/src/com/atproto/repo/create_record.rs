// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `com.atproto.repo.createRecord` namespace."]
#[doc = "`com.atproto.repo.createRecord`"]
#[doc = "Create a new record."]
#[async_trait::async_trait]
pub trait CreateRecord: crate::xrpc::XrpcClient {
    async fn create_record(&self, input: Input) -> Result<Output, crate::xrpc::Error<Error>> {
        let body = crate::xrpc::XrpcClient::send(
            self,
            http::Method::POST,
            "com.atproto.repo.createRecord",
            None,
            Some(serde_json::to_vec(&input)?),
            Some(String::from("application/json")),
        )
        .await?;
        serde_json::from_slice(&body).map_err(|e| e.into())
    }
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    #[doc = "The NSID of the record collection."]
    pub collection: String,
    #[doc = "The record to create."]
    pub record: crate::records::Record,
    #[doc = "The handle or DID of the repo."]
    pub repo: String,
    #[doc = "The key of the record."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rkey: Option<String>,
    #[doc = "Compare and swap with the previous commit by cid."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_commit: Option<String>,
    #[doc = "Validate the record?"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate: Option<bool>,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub cid: String,
    pub uri: String,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {
    InvalidSwap(Option<String>),
}
