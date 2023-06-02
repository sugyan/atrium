// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `com.atproto.server.revokeAppPassword` namespace."]
#[doc = "`com.atproto.server.revokeAppPassword`"]
#[doc = "Revoke an app-specific password by name."]
#[async_trait::async_trait]
pub trait RevokeAppPassword: crate::xrpc::XrpcClient {
    async fn revoke_app_password(&self, input: Input) -> Result<(), crate::xrpc::Error<Error>> {
        let _ = crate::xrpc::XrpcClient::send(
            self,
            http::Method::POST,
            "com.atproto.server.revokeAppPassword",
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
    pub name: String,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {}
