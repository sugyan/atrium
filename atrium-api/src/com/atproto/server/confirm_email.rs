// This file is generated by atrium-codegen. DO NOT EDIT.
//!Definitions for the `com.atproto.server.confirmEmail` namespace.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub email: String,
    pub token: String,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {
    AccountNotFound(Option<String>),
    ExpiredToken(Option<String>),
    InvalidToken(Option<String>),
    InvalidEmail(Option<String>),
}
