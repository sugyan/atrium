// This file is generated by atrium-codegen. DO NOT EDIT.
//!Definitions for the `app.bsky.labeler.getServices` namespace.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed: Option<bool>,
    pub dids: Vec<crate::types::string::Did>,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub views: Vec<crate::types::Union<OutputViewsItem>>,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Ok(())
    }
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "$type")]
pub enum OutputViewsItem {
    #[serde(rename = "app.bsky.labeler.defs#labelerView")]
    AppBskyLabelerDefsLabelerView(Box<crate::app::bsky::labeler::defs::LabelerView>),
    #[serde(rename = "app.bsky.labeler.defs#labelerViewDetailed")]
    AppBskyLabelerDefsLabelerViewDetailed(
        Box<crate::app::bsky::labeler::defs::LabelerViewDetailed>,
    ),
}
