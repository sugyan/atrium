// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `com.atproto.server.getAccountInviteCodes` namespace."]
#[doc = "`com.atproto.server.getAccountInviteCodes`"]
#[doc = "Get all invite codes for a given account"]
#[async_trait::async_trait]
pub trait GetAccountInviteCodes: crate::xrpc::XrpcClient {
    async fn get_account_invite_codes(
        &self,
        params: Parameters,
    ) -> Result<Output, crate::xrpc::Error<Error>> {
        let body = crate::xrpc::XrpcClient::send(
            self,
            http::Method::GET,
            "com.atproto.server.getAccountInviteCodes",
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
    pub create_available: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_used: Option<bool>,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub codes: Vec<crate::com::atproto::server::defs::InviteCode>,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {
    DuplicateCreate(Option<String>),
}
