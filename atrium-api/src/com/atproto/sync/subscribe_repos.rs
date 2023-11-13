// This file is generated by atrium-codegen. DO NOT EDIT.
//!Definitions for the `com.atproto.sync.subscribeRepos` namespace.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    ///The last known event to backfill from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<i32>,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {
    FutureCursor(Option<String>),
    ConsumerTooSlow(Option<String>),
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub blobs: Vec<cid::Cid>,
    ///CAR file containing relevant blocks
    #[serde(with = "serde_bytes")]
    pub blocks: Vec<u8>,
    pub commit: cid::Cid,
    pub ops: Vec<RepoOp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<cid::Cid>,
    pub rebase: bool,
    pub repo: String,
    ///The rev of the emitted commit
    pub rev: String,
    pub seq: i32,
    ///The rev of the last emitted commit from this repo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    pub time: String,
    pub too_big: bool,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Handle {
    pub did: String,
    pub handle: String,
    pub seq: i32,
    pub time: String,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub name: String,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Migrate {
    pub did: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migrate_to: Option<String>,
    pub seq: i32,
    pub time: String,
}
///A repo operation, ie a write of a single record. For creates and updates, cid is the record's CID as of this operation. For deletes, it's null.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepoOp {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<cid::Cid>,
    pub path: String,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Tombstone {
    pub did: String,
    pub seq: i32,
    pub time: String,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "$type")]
pub enum Message {
    #[serde(rename = "com.atproto.sync.subscribeRepos#commit")]
    Commit(Box<Commit>),
    #[serde(rename = "com.atproto.sync.subscribeRepos#handle")]
    Handle(Box<Handle>),
    #[serde(rename = "com.atproto.sync.subscribeRepos#migrate")]
    Migrate(Box<Migrate>),
    #[serde(rename = "com.atproto.sync.subscribeRepos#tombstone")]
    Tombstone(Box<Tombstone>),
    #[serde(rename = "com.atproto.sync.subscribeRepos#info")]
    Info(Box<Info>),
}
