// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `app.bsky.actor.putPreferences` namespace."]
#[doc = "`app.bsky.actor.putPreferences`"]
#[doc = "Sets the private preferences attached to the account."]
#[async_trait::async_trait]
pub trait PutPreferences: crate::xrpc::XrpcClient {
    async fn put_preferences(&self, input: Input) -> Result<(), crate::xrpc::Error<Error>> {
        let _ = crate::xrpc::XrpcClient::send(
            self,
            http::Method::POST,
            "app.bsky.actor.putPreferences",
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
    pub preferences: crate::app::bsky::actor::defs::Preferences,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {}
