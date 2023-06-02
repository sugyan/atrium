// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `com.atproto.server.refreshSession` namespace."]
#[doc = "`com.atproto.server.refreshSession`"]
#[doc = "Refresh an authentication session."]
#[async_trait::async_trait]
pub trait RefreshSession: crate::xrpc::XrpcClient {
    async fn refresh_session(&self) -> Result<Output, crate::xrpc::Error<Error>> {
        let body = crate::xrpc::XrpcClient::send(
            self,
            http::Method::POST,
            "com.atproto.server.refreshSession",
            None,
            None,
            None,
        )
        .await?;
        serde_json::from_slice(&body).map_err(|e| e.into())
    }
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub access_jwt: String,
    pub did: String,
    pub handle: String,
    pub refresh_jwt: String,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {
    AccountTakedown(Option<String>),
}
