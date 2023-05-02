// This file is generated by atrium-codegen. Do not edit.
//! Definitions for the `com.atproto.admin.resolveModerationReports` namespace.

/// Resolve moderation reports by an action.
#[async_trait::async_trait]
pub trait ResolveModerationReports: crate::xrpc::XrpcClient {
    async fn resolve_moderation_reports(&self, input: Input) -> Result<Output, Box<dyn std::error::Error>> {
        let body = crate::xrpc::XrpcClient::send::<Error>(
            self,
            http::Method::POST,
            "com.atproto.admin.resolveModerationReports",
            None,
            Some(serde_json::to_vec(&input)?),
            Some(String::from("application/json")),
        )
        .await?;
        serde_json::from_slice(&body).map_err(|e| e.into())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub action_id: i32,
    pub created_by: String,
    pub report_ids: Vec<i32>,
}

pub type Output = crate::com::atproto::admin::defs::ActionView;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {
}