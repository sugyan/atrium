// This file is generated by atrium-codegen. Do not edit.
//! Definitions for the `com.atproto.sync.getRecord` namespace.

/// Gets blocks needed for existence or non-existence of record.
#[async_trait::async_trait]
pub trait GetRecord: crate::xrpc::XrpcClient {
    async fn get_record(&self, params: Parameters) -> Result<(), Box<dyn std::error::Error>> {
        crate::xrpc::XrpcClient::send_unit(
            self,
            http::Method::GET,
            "com.atproto.sync.getRecord",
            Some(params),
            Option::<()>::None,
        )
        .await
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    pub collection: String,
    /// An optional past commit CID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// The DID of the repo.
    pub did: String,
    pub rkey: String,
}


pub enum Error {
}
