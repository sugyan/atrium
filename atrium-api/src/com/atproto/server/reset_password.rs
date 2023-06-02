// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `com.atproto.server.resetPassword` namespace."]
#[doc = "`com.atproto.server.resetPassword`"]
#[doc = "Reset a user account password using a token."]
#[async_trait::async_trait]
pub trait ResetPassword: crate::xrpc::XrpcClient {
    async fn reset_password(&self, input: Input) -> Result<(), crate::xrpc::Error<Error>> {
        let _ = crate::xrpc::XrpcClient::send(
            self,
            http::Method::POST,
            "com.atproto.server.resetPassword",
            None,
            Some(serde_json::to_vec(&input)?),
            Some(String::from("application/json")),
        )
        .await?;
        Ok(())
    }
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub password: String,
    pub token: String,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {
    ExpiredToken(Option<String>),
    InvalidToken(Option<String>),
}
