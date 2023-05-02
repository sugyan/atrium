// This file is generated by atrium-codegen. Do not edit.
//! Definitions for the `app.bsky.graph.follow` namespace.

// app.bsky.graph.follow
/// A social follow.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub created_at: String,
    pub subject: String,
}