// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `app.bsky.actor.searchActorsTypeahead` namespace."]
#[doc = "`app.bsky.actor.searchActorsTypeahead`"]
#[doc = "Find actor suggestions for a search term."]
#[async_trait::async_trait]
pub trait SearchActorsTypeahead: crate::xrpc::XrpcClient {
    async fn search_actors_typeahead(
        &self,
        params: Parameters,
    ) -> Result<Output, crate::xrpc::Error<Error>> {
        let body = crate::xrpc::XrpcClient::send(
            self,
            http::Method::GET,
            "app.bsky.actor.searchActorsTypeahead",
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
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term: Option<String>,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub actors: Vec<crate::app::bsky::actor::defs::ProfileViewBasic>,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {}
