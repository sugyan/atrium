// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `com.atproto.admin.getRecord` namespace."]
#[doc = "`com.atproto.admin.getRecord`"]
#[doc = "View details about a record."]
#[async_trait::async_trait]
pub trait GetRecord: crate::xrpc::XrpcClient {
    async fn get_record(&self, params: Parameters) -> Result<Output, crate::xrpc::Error<Error>> {
        let body = crate::xrpc::XrpcClient::send(
            self,
            http::Method::GET,
            "com.atproto.admin.getRecord",
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    pub uri: String,
}
pub type Output = crate::com::atproto::admin::defs::RecordViewDetail;
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {
    RecordNotFound(Option<String>),
}
