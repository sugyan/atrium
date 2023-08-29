// This file is generated by atrium-codegen. DO NOT EDIT.
#![doc = "Definitions for the `com.atproto.admin.takeModerationAction` namespace."]
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_label_vals: Option<Vec<String>>,
    pub created_by: String,
    #[doc = "Indicates how long this action was meant to be in effect before automatically expiring."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_in_hours: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negate_label_vals: Option<Vec<String>>,
    pub reason: String,
    pub subject: InputSubjectEnum,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_blob_cids: Option<Vec<String>>,
}
pub type Output = crate::com::atproto::admin::defs::ActionView;
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {
    SubjectHasAction(Option<String>),
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "$type")]
pub enum InputSubjectEnum {
    #[serde(rename = "com.atproto.admin.defs#repoRef")]
    ComAtprotoAdminDefsRepoRef(Box<crate::com::atproto::admin::defs::RepoRef>),
    #[serde(rename = "com.atproto.repo.strongRef")]
    ComAtprotoRepoStrongRefMain(Box<crate::com::atproto::repo::strong_ref::Main>),
}
