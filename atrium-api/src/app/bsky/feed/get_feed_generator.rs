// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `app.bsky.feed.getFeedGenerator` namespace."]
#[doc = "`app.bsky.feed.getFeedGenerator`"]
#[doc = "Get information about a specific feed offered by a feed generator, such as its online status"]
#[async_trait::async_trait]
pub trait GetFeedGenerator: crate::xrpc::XrpcClient {
    async fn get_feed_generator(
        &self,
        params: Parameters,
    ) -> Result<Output, crate::xrpc::Error<Error>> {
        let body = crate::xrpc::XrpcClient::send(
            self,
            http::Method::GET,
            "app.bsky.feed.getFeedGenerator",
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
    pub feed: String,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub is_online: bool,
    pub is_valid: bool,
    pub view: crate::app::bsky::feed::defs::GeneratorView,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {}
