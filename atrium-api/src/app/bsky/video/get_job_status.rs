// This file is generated by atrium-codegen. DO NOT EDIT.
//!Definitions for the `app.bsky.video.getJobStatus` namespace.
pub const NSID: &str = "app.bsky.video.getJobStatus";
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParametersData {
    pub job_id: String,
}
pub type Parameters = crate::types::Object<ParametersData>;
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OutputData {
    pub job_status: crate::app::bsky::video::defs::JobStatus,
}
pub type Output = crate::types::Object<OutputData>;
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Ok(())
    }
}