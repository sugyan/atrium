// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `app.bsky.graph.muteActor` namespace."]
#[doc = "`app.bsky.graph.muteActor`"]
#[doc = "Mute an actor by did or handle."]
#[async_trait::async_trait]
pub trait MuteActor: crate::xrpc::XrpcClient {
    async fn mute_actor(&self, input: Input) -> Result<(), crate::xrpc::Error<Error>> {
        let _ = crate::xrpc::XrpcClient::send(
            self,
            http::Method::POST,
            "app.bsky.graph.muteActor",
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
    pub actor: String,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {}
