// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `com.atproto.repo.getRecord` namespace."]
#[doc = "`com.atproto.repo.getRecord`"]
#[doc = "Get a record."]
#[async_trait::async_trait]
pub trait GetRecord: crate::xrpc::XrpcClient {
    async fn get_record(&self, params: Parameters) -> Result<Output, crate::xrpc::Error<Error>> {
        let body = crate::xrpc::XrpcClient::send(
            self,
            http::Method::GET,
            "com.atproto.repo.getRecord",
            Some(serde_urlencoded::to_string(&params)?),
            None,
            None,
        )
        .await?;
        serde_json::from_slice(&body).map_err(|e| e.into())
    }
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    #[doc = "The CID of the version of the record. If not specified, then return the most recent version."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    #[doc = "The NSID of the record collection."]
    pub collection: String,
    #[doc = "The handle or DID of the repo."]
    pub repo: String,
    #[doc = "The key of the record."]
    pub rkey: String,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    pub uri: String,
    pub value: crate::records::Record,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {}
