// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `com.atproto.moderation.createReport` namespace."]
#[doc = "`com.atproto.moderation.createReport`"]
#[doc = "Report a repo or a record."]
#[async_trait::async_trait]
pub trait CreateReport: crate::xrpc::XrpcClient {
    async fn create_report(&self, input: Input) -> Result<Output, crate::xrpc::Error<Error>> {
        let body = crate::xrpc::XrpcClient::send(
            self,
            http::Method::POST,
            "com.atproto.moderation.createReport",
            None,
            Some(serde_json::to_vec(&input)?),
            Some(String::from("application/json")),
        )
        .await?;
        serde_json::from_slice(&body).map_err(|e| e.into())
    }
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub reason_type: crate::com::atproto::moderation::defs::ReasonType,
    pub subject: InputSubjectEnum,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub created_at: String,
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub reason_type: crate::com::atproto::moderation::defs::ReasonType,
    pub reported_by: String,
    pub subject: OutputSubjectEnum,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "$type")]
pub enum InputSubjectEnum {
    #[serde(rename = "com.atproto.admin.defs#repoRef")]
    ComAtprotoAdminDefsRepoRef(Box<crate::com::atproto::admin::defs::RepoRef>),
    #[serde(rename = "com.atproto.repo.strongRef")]
    ComAtprotoRepoStrongRefMain(Box<crate::com::atproto::repo::strong_ref::Main>),
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "$type")]
pub enum OutputSubjectEnum {
    #[serde(rename = "com.atproto.admin.defs#repoRef")]
    ComAtprotoAdminDefsRepoRef(Box<crate::com::atproto::admin::defs::RepoRef>),
    #[serde(rename = "com.atproto.repo.strongRef")]
    ComAtprotoRepoStrongRefMain(Box<crate::com::atproto::repo::strong_ref::Main>),
}
